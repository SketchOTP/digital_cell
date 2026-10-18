#!/usr/bin/env python3
"""Illustrative prescribed-stroke scalar reference; no native biology integration."""
from __future__ import annotations

import argparse
from dataclasses import dataclass, fields, is_dataclass
from fractions import Fraction as F
import json
from pathlib import Path
from typing import Any


class Rejected(ValueError):
    """The complete proposed cycle was rejected; its input Run is unchanged."""


def rational(value: Any, name: str, *, positive: bool = False,
             nonnegative: bool = False) -> F:
    if type(value) is not F:
        raise Rejected(f"{name}: exact Fraction required")
    if positive and value <= 0 or nonnegative and value < 0:
        raise Rejected(f"{name}: invalid sign")
    return value


@dataclass(frozen=True)
class Params:
    K: F
    lam: F
    theta: F  # Owned typed parameter; no exponential kinetic law runs here.
    epsilon_A: F
    k_s: F
    gamma: F
    c0: F
    h: F

    def __post_init__(self) -> None:
        for item in fields(self):
            rational(getattr(self, item.name), item.name, positive=True)


@dataclass(frozen=True)
class Material:
    P: F
    M: F
    rho_s: F
    phi: F

    def __post_init__(self) -> None:
        for item in fields(self):
            rational(getattr(self, item.name), item.name, positive=True)
        if self.rho_s < F(1, 10**15) or self.L0 < F(1, 1000) or self.phi > 1:
            raise Rejected("unsupported scaffold domain")

    @property
    def L0(self) -> F:
        return self.M / self.rho_s


@dataclass(frozen=True)
class Accuracy:
    force: F
    width: F
    step_energy: F
    cumulative_energy: F
    max_iterations: int

    def __post_init__(self) -> None:
        for name in ("force", "width", "step_energy"):
            rational(getattr(self, name), name, positive=True)
        rational(self.cumulative_energy, "cumulative_energy", nonnegative=True)
        if type(self.max_iterations) is not int or not 1 <= self.max_iterations <= 4096:
            raise Rejected("max_iterations must be an integer in [1,4096]")


def _geometry(material: Material, length: F) -> None:
    rational(length, "L", positive=True)
    if length > 4 * material.L0:
        raise Rejected("length outside owned domain")


def coupling(params: Params, material: Material, a: F, length: F) -> F:
    """Local algebra only: calling this does not establish state reachability."""
    _geometry(material, length)
    rational(a, "a", nonnegative=True)
    if a > material.P:
        raise Rejected("polarity donor domain")
    return params.K * material.P / 2 * (
        (length - material.L0) / material.L0 - params.lam * a / material.P
    ) ** 2


def stretch(params: Params, material: Material, length: F) -> F:
    _geometry(material, length)
    factor = F(1) if length <= material.L0 else material.phi
    return factor * params.k_s * (length - material.L0) ** 2 / (2 * material.L0)


def stretch_secant(params: Params, material: Material, old: F, new: F) -> F:
    _geometry(material, old)
    _geometry(material, new)
    if new == old:
        factor = F(1) if old <= material.L0 else material.phi
        return factor * params.k_s * (old - material.L0) / material.L0
    # Exact Fraction division is the true piecewise potential secant, including
    # crossings of L0. There is no cancellation error or branch shortcut.
    return (stretch(params, material, new) - stretch(params, material, old)) / (new - old)


def reaction_variations(params: Params, material: Material, a: F,
                        length: F, d: F) -> tuple[F, F]:
    """Total uphill/downhill work of one monotone prescribed conversion stroke."""
    rational(d, "d")
    start = coupling(params, material, a, length)
    end = coupling(params, material, a + d, length)
    xi_star = material.P * (length - material.L0) / (material.L0 * params.lam)
    low, high = sorted((a, a + d))
    minimum = F(0) if low <= xi_star <= high else min(start, end)
    return end - minimum, start - minimum


def residual(params: Params, material: Material, a: F, old: F, new: F) -> F:
    _geometry(material, old)
    _geometry(material, new)
    rational(a, "a", nonnegative=True)
    if a > material.P:
        raise Rejected("polarity donor domain")
    tc = params.K * material.P / material.L0 * (
        (new + old) / (2 * material.L0) - 1 - params.lam * a / material.P
    )
    return params.gamma * (new - old) / params.h + tc + stretch_secant(
        params, material, old, new)


@dataclass(frozen=True)
class Certificate:
    lo: F
    hi: F
    f_lo: F
    f_hi: F
    root: F
    force_residual: F
    width: F
    energy_error: F
    iterations: int


def solve_mechanics(params: Params, material: Material, a: F, old: F,
                    accuracy: Accuracy, bracket: tuple[F, F]) -> Certificate:
    if type(bracket) is not tuple or len(bracket) != 2:
        raise Rejected("bracket must be an exact two-element tuple")
    lo, hi = bracket
    _geometry(material, lo)
    _geometry(material, hi)
    if lo > hi:
        raise Rejected("reversed bracket")
    f_lo = residual(params, material, a, old, lo)
    f_hi = residual(params, material, a, old, hi)
    if f_lo > 0 or f_hi < 0:
        raise Rejected("uncertified root bracket signs")
    if f_lo == 0:
        hi, f_hi = lo, f_lo
    elif f_hi == 0:
        lo, f_lo = hi, f_hi
    for iteration in range(1, accuracy.max_iterations + 1):
        root = (lo + hi) / 2
        force_error = residual(params, material, a, old, root)
        if force_error == 0:
            lo = hi = root
            f_lo = f_hi = F(0)
        width = hi - lo
        error = force_error * (root - old)
        if (width <= accuracy.width and abs(force_error) <= accuracy.force
                and abs(error) <= accuracy.step_energy):
            return Certificate(lo, hi, f_lo, f_hi, root, force_error,
                               width, error, iteration)
        if force_error < 0:
            lo, f_lo = root, force_error
        else:
            hi, f_hi = root, force_error
    raise Rejected("finite mechanics iteration cap exhausted")


@dataclass(frozen=True)
class State:
    a: F
    b: F
    A: F
    W: F
    L: F
    Q_chem: F
    Q_drag: F
    absolute_energy_error: F
    signed_energy_error: F
    cycles: int


@dataclass(frozen=True, init=False)
class Run:
    params: Params
    material: Material
    accuracy: Accuracy
    state: State
    initial_A: F
    initial_W: F
    certificates: tuple[Certificate, ...]

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        raise Rejected("Run entry is cold() only; imported states are unsupported")

    @staticmethod
    def _publish(params: Params, material: Material, accuracy: Accuracy,
                 state: State, initial_A: F, initial_W: F,
                 certificates: tuple[Certificate, ...]) -> Run:
        run = object.__new__(Run)
        for key, value in zip(("params", "material", "accuracy", "state", "initial_A",
                               "initial_W", "certificates"),
                              (params, material, accuracy, state, initial_A,
                               initial_W, certificates)):
            object.__setattr__(run, key, value)
        return run

    @classmethod
    def cold(cls, params: Params, material: Material, accuracy: Accuracy,
             A: F, W: F = F(0)) -> Run:
        if type(params) is not Params or type(material) is not Material or type(accuracy) is not Accuracy:
            raise Rejected("owned fixed Params, Material and Accuracy required")
        rational(A, "A", nonnegative=True)
        rational(W, "W", nonnegative=True)
        state = State(F(0), material.P, A, W, material.L0,
                      F(0), F(0), F(0), F(0), 0)
        return cls._publish(params, material, accuracy, state, A, W, ())

    def cycle(self, d: F, bracket: tuple[F, F]) -> Run:
        rational(d, "d")
        s, p, m = self.state, self.params, self.material
        if not -s.a <= d <= s.b:
            raise Rejected("insufficient polarity donor")
        up, down = reaction_variations(p, m, s.a, s.L, d)
        z = p.c0 * abs(d) + up / p.epsilon_A
        if z > s.A:
            raise Rejected("insufficient pre-stroke finite A")
        # All reaction results remain scratch while actual mechanics is solved.
        a, b, A, W = s.a + d, s.b - d, s.A - z, s.W + z
        q_chem = p.epsilon_A * p.c0 * abs(d) + down
        cert = solve_mechanics(p, m, a, s.L, self.accuracy, bracket)
        q_drag = p.gamma * (cert.root - s.L) ** 2 / p.h
        actual_error = (coupling(p, m, a, cert.root) + stretch(p, m, cert.root)
                        - coupling(p, m, a, s.L) - stretch(p, m, s.L) + q_drag)
        if actual_error != cert.energy_error:
            raise Rejected("discrete work identity defect")
        absolute = s.absolute_energy_error + abs(actual_error)
        if absolute > self.accuracy.cumulative_energy:
            raise Rejected("cumulative absolute energy-error budget exhausted")
        state = State(a, b, A, W, cert.root, s.Q_chem + q_chem,
                      s.Q_drag + q_drag, absolute,
                      s.signed_energy_error + actual_error, s.cycles + 1)
        if a + b != m.P or A + W != self.initial_A + self.initial_W:
            raise Rejected("material conservation defect")
        total_error = (p.epsilon_A * A + coupling(p, m, a, cert.root)
                       + stretch(p, m, cert.root) + state.Q_chem + state.Q_drag
                       - p.epsilon_A * self.initial_A)
        if total_error != state.signed_energy_error or abs(total_error) > absolute:
            raise Rejected("whole-run energy certificate defect")
        return self._publish(p, m, self.accuracy, state, self.initial_A,
                             self.initial_W, self.certificates + (cert,))

    def unsupported(self, operation: str) -> None:
        raise Rejected(f"unsupported lifecycle, port, parameter change or OFF operation: {operation}")


def jsonable(value: Any) -> Any:
    if type(value) is F:
        return str(value)
    if is_dataclass(value):
        return {item.name: jsonable(getattr(value, item.name)) for item in fields(value)}
    if isinstance(value, (list, tuple)):
        return [jsonable(item) for item in value]
    if isinstance(value, dict):
        return {key: jsonable(item) for key, item in value.items()}
    return value


def demo() -> dict[str, Any]:
    """Illustrative constants and prescribed strokes; these are not calibration."""
    p = Params(F(2), F(1, 3), F(1), F(3), F(1), F(1), F(1, 2), F(1, 10))
    m = Material(F(1), F(1), F(1), F(1, 2))
    accuracy = Accuracy(F(1, 10**9), F(1, 10**10), F(1, 10**10), F(1, 10**8), 80)
    run = Run.cold(p, m, accuracy, F(10))
    strokes = (F(1, 4), F(1, 4), F(-1, 8), F(-1, 4))
    history = [run]
    for d in strokes:
        run = run.cycle(d, (F(1, 10), F(4)))
        history.append(run)
    return {"classification": "ILLUSTRATIVE_BOUNDED_REFERENCE_ONLY",
            "calibrated": False, "native_J_trajectory": False,
            "prescribed_strokes": strokes, "history": history}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="create new JSON file; never overwrite")
    args = parser.parse_args()
    payload = json.dumps(jsonable(demo()), sort_keys=True, indent=2) + "\n"
    if args.output is None:
        print(payload, end="")
    else:
        with args.output.open("x", encoding="utf-8") as handle:
            handle.write(payload)


if __name__ == "__main__":
    main()
