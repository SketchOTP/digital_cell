#!/usr/bin/env python3
"""Conditional exact unit/work maps; no acquired code, data fit or calibration."""
from dataclasses import dataclass, fields
from fractions import Fraction as F


def exact(value, name="value", positive=False):
    if type(value) is not F or positive and value <= 0:
        raise ValueError(f"{name}: exact {'positive ' if positive else ''}Fraction required")
    return value


def require(condition, message):
    if not condition:
        raise ValueError(message)


@dataclass(frozen=True)
class Scales:
    ell: F
    tau: F
    p: F
    f: F
    e: F

    def __post_init__(self):
        for field in fields(self):
            exact(getattr(self, field.name), field.name, True)

    def factors(self):
        return {"L": self.ell, "L0": self.ell, "t": self.tau, "h": self.tau,
                "a": self.p, "b": self.p, "P": self.p, "A": self.f, "W": self.f,
                "energy": self.e, "heat": self.e, "work": self.e,
                "force": self.e / self.ell, "ks": self.e / self.ell,
                "K": self.e / self.p, "theta": self.e / self.p,
                "g": self.e / self.p, "epsilon_A": self.e / self.f,
                "lambda": F(1), "gamma": self.e * self.tau / self.ell**2,
                "c0": self.f / self.p, "diffusion": self.ell**2 / self.tau,
                "k0": 1 / self.tau, "kd": 1 / self.tau,
                "kf": self.ell**2 / (self.tau * self.p**2),
                "kq": self.ell**2 / (self.tau * self.p**2)}

    def to_source(self, quantity, value):
        exact(value)
        require(quantity in self.factors(), "unknown mapped quantity")
        return self.factors()[quantity] * value


def coupling(K, lam, P, L0, L, a):
    for name, value in (("K", K), ("lambda", lam), ("P", P), ("L0", L0)):
        exact(value, name, True)
    exact(L, "L", True)
    exact(a, "a")
    require(0 <= a <= P and L <= 4 * L0, "outside fixed-material boundary")
    return K * P / 2 * ((L - L0) / L0 - lam * a / P)**2


def efforts(K, lam, P, L0, L, a):
    coupling(K, lam, P, L0, L, a)
    z = (L - L0) / L0 - lam * a / P
    return K * P / L0 * z, -K * lam * z


@dataclass(frozen=True)
class TwoPortFixture:
    """Designed fixed-material fixture; constructor values are NOT observations."""
    kref: F
    sL: F
    sa: F
    e: F
    P: F
    L0: F

    def __post_init__(self):
        for field in fields(self):
            exact(getattr(self, field.name), field.name, True)

    @property
    def K(self):
        return self.kref * self.sL**2 * self.L0**2 / (self.e * self.P)

    @property
    def lam(self):
        return self.sa * self.P / (self.sL * self.L0)

    def extension(self, L, a):
        coupling(self.K, self.lam, self.P, self.L0, L, a)
        return self.sL * (L - self.L0) - self.sa * a

    def energy(self, L, a):
        return self.kref * self.extension(L, a)**2 / 2

    def target_efforts(self, L, a):
        extension = self.extension(L, a)
        return (self.sL * self.kref * extension / self.e,
                -self.sa * self.kref * extension / self.e)

    def discrete_work(self, old_L, old_a, new_L, new_a):
        old, new = self.extension(old_L, old_a), self.extension(new_L, new_a)
        effort = self.kref * (old + new) / (2 * self.e)
        return (effort * self.sL * (new_L - old_L),
                -effort * self.sa * (new_a - old_a))


def rotary_chart(angle, torque, gamma_rot, radius):
    for name, value in (("angle", angle), ("torque", torque)):
        exact(value, name)
    exact(gamma_rot, "gamma_rot", True)
    exact(radius, "radius", True)
    # Radian branch only. No physical lever arm or winding-to-edge identity.
    return radius * angle, torque / radius, gamma_rot / radius**2


def binary_mixture(N, k, d, q, x):
    for name, value in (("N", N), ("k", k), ("d", d)):
        exact(value, name, True)
    exact(q, "q")
    exact(x, "x")
    require(0 <= x <= 1, "occupancy outside [0,1]")
    mean = N * ((1 - x) * k * q**2 / 2 + x * k * (q - d)**2 / 2)
    shifted = N * k * (q - d * x)**2 / 2
    variance = N * k * d**2 * x * (1 - x) / 2
    return mean, shifted, variance


@dataclass(frozen=True)
class FiniteFuel:
    """Conditional isolated IDEAL-mixture derivative, not acquired motor data."""
    nT: F
    nD: F
    nP: F
    kBT: F

    def __post_init__(self):
        for field in fields(self):
            exact(getattr(self, field.name), field.name, True)

    def derivative(self, xi):
        exact(xi, "xi")
        require(0 <= xi < self.nT, "fuel depletion domain")
        return -self.kBT * (1 / (self.nT - xi) + 1 / (self.nD + xi)
                           + 1 / (self.nP + xi))

    def local_bounds(self, X):
        exact(X, "X", True)
        require(X < self.nT, "bound requires X<nT")
        B = self.kBT * (1 / (self.nT - X) + 1 / self.nD + 1 / self.nP)
        return {"derivative_bound": B, "affinity_error_bound": B * X,
                "integrated_energy_error_bound": B * X**2 / 2}


def evaluate_reference_checks():
    """Deterministic algebraic controls, with no admitted data or fitted values."""
    counts = {"similarity_cases": 0, "two_port_cases": 0,
              "rotary_cases": 0, "mixture_cases": 0, "fuel_bound_cases": 0}
    for s in (Scales(F(2), F(3), F(5), F(7), F(11)),
              Scales(F(3, 2), F(2, 3), F(7, 3), F(5, 4), F(13, 5))):
        K, lam, P, L0 = F(3), F(2, 5), F(2), F(3)
        for L, a in ((F(2), F(0)), (F(3), F(1)), (F(4), F(3, 2))):
            source = [s.to_source(key, value) for key, value in
                      zip(("K", "lambda", "P", "L0", "L", "a"), (K, lam, P, L0, L, a))]
            require(coupling(*source) == s.e * coupling(K, lam, P, L0, L, a), "energy similarity")
            target_T, target_g = efforts(K, lam, P, L0, L, a)
            source_T, source_g = efforts(*source)
            require(source_T == s.e / s.ell * target_T and source_g == s.e / s.p * target_g,
                    "conjugate effort similarity")
            require(source_T * s.ell * F(1, 3) == s.e * target_T * F(1, 3), "mechanical work")
            require(source_g * s.p * F(-1, 4) == s.e * target_g * F(-1, 4), "chemical work")
            f = s.factors()
            require(f["gamma"] * s.ell / s.tau == f["force"], "drag similarity")
            require(f["epsilon_A"] * s.f == s.e, "source energy similarity")
            require(f["c0"] * s.p == s.f, "overhead material similarity")
            require(f["kf"] * s.p**3 / s.ell**2 == s.p / s.tau, "nonlinear rate similarity")
            require(f["k0"] * s.p == s.p / s.tau, "linear rate similarity")
            require(f["g"] / f["theta"] == 1, "kinetic tilt similarity")
            # Complete prescribed-stroke variation ledger, including uphill
            # storage and irreversible downhill heat; not endpoint-only work.
            c0, d, up, down, epsA = F(1, 2), F(-1, 4), F(3, 4), F(1, 5), F(5)
            cost = c0 * abs(d) + up / epsA
            source_cost = f["c0"] * c0 * abs(s.p * d) + s.e * up / (f["epsilon_A"] * epsA)
            source_heat = f["epsilon_A"] * epsA * f["c0"] * c0 * abs(s.p * d) + s.e * down
            require(source_cost == s.f * cost, "finite stroke fuel ledger similarity")
            require(source_heat == s.e * (epsA * c0 * abs(d) + down), "chemical heat similarity")
            require(-f["epsilon_A"] * epsA * source_cost + s.e * (up - down)
                    + source_heat == 0, "source-store-heat energy identity")
            counts["similarity_cases"] += 1
    for e in (F(1), F(7, 3)):
        fixture = TwoPortFixture(F(5), F(3, 2), F(2, 3), e, F(2), F(3))
        for L, a in ((F(2), F(0)), (F(3), F(1)), (F(4), F(3, 2))):
            require(fixture.energy(L, a) == e * coupling(fixture.K, fixture.lam, fixture.P,
                                                       fixture.L0, L, a), "two-port energy")
            require(fixture.target_efforts(L, a) == efforts(fixture.K, fixture.lam,
                    fixture.P, fixture.L0, L, a), "two-port Jacobian")
            work = sum(fixture.discrete_work(F(2), F(0), L, a), F(0))
            require(work == (fixture.energy(L, a) - fixture.energy(F(2), F(0))) / e,
                    "finite two-port work")
            counts["two_port_cases"] += 1
    for angle, torque in ((F(2, 3), F(5)), (F(-3, 4), F(2)), (F(1), F(-2))):
        q, force, drag = rotary_chart(angle, torque, F(7), F(3))
        require(q * force == angle * torque and drag * F(3)**2 == 7, "rotary work/drag")
        counts["rotary_cases"] += 1
    for x in (F(0), F(1, 4), F(1, 2), F(3, 4), F(1)):
        mean, shifted, variance = binary_mixture(F(3), F(2), F(4), F(5), x)
        require(mean - shifted == variance, "binary mixture hidden energy")
        counts["mixture_cases"] += 1
    fuel = FiniteFuel(F(10), F(3), F(4), F(2))
    bound = fuel.local_bounds(F(2))
    for xi in (F(0), F(1, 2), F(1), F(3, 2), F(2)):
        require(0 < -fuel.derivative(xi) <= bound["derivative_bound"], "finite-source bound")
        counts["fuel_bound_cases"] += 1
    return {"counts": counts, "total_cases": sum(counts.values()),
            "algebraic_checks_passed": True, "full_calibration_established": False,
            "source_species_mapping_established": False, "fixture_built_or_measured": False,
            "admitted_target_observations": 0, "parameter_assignment_performed": False,
            "scope": "CONDITIONAL_ALGEBRA_AND_COUNTEREXAMPLES_ONLY"}
