#!/usr/bin/env python3
"""Published closed CRN plus explicitly NEW, uncalibrated coupled reference."""
from dataclasses import asdict, dataclass
import math

SPECIES = ("E", "ES", "ESS", "Pr", "S", "Sm")
PUBLISHED_INITIAL = (0.1, 0.05, 0.05, 0.01, 1.0, 0.1)
NU_A = (0, 1, 0, -1)
THETA = 1.0  # Normalized chemical energy/amount, not measured ATP or target theta.
STOICH = ((0, -1, 0, 1), (0, 1, -1, 0), (0, 0, 1, -1),
          (0, 0, 0, 1), (-1, 0, 0, 0), (1, -1, -1, 0))


class Rejected(ValueError):
    pass


def number(value, name, *, positive=False, nonnegative=False):
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise Rejected(f"{name}: finite real number required")
    if positive and value <= 0 or nonnegative and value < 0:
        raise Rejected(f"{name}: invalid sign")
    return float(value)


@dataclass(frozen=True)
class Config:
    K: float = 0.0
    lam: float = 0.0
    k_s: float = 1.0
    L0: float = 1.0
    gamma: float = 1.0
    phi: float = 1.0
    rate_scale: float = 1.0
    max_halvings: int = 12
    max_steps: int = 100000
    monotone_tolerance: float = 1e-12

    def __post_init__(self):
        for name in ("K", "lam", "rate_scale", "monotone_tolerance"):
            number(getattr(self, name), name, nonnegative=True)
        for name in ("k_s", "L0", "gamma", "phi"):
            number(getattr(self, name), name, positive=True)
        if self.phi > 1:
            raise Rejected("phi must not exceed one")
        for name, cap in (("max_halvings", 30), ("max_steps", 1000000)):
            value = getattr(self, name)
            if type(value) is not int or not 1 <= value <= cap:
                raise Rejected(f"{name}: invalid finite integer cap")


def species(values):
    if type(values) not in (list, tuple) or len(values) != 6:
        raise Rejected("six positive species concentrations required")
    return tuple(number(value, name, positive=True) for name, value in zip(SPECIES, values))


def totals(z):
    E, ES, ESS, Pr, S, Sm = species(z)
    return E + ES + ESS, S + Sm + ES + 2 * ESS + 2 * Pr


def bare_rates(z):
    E, ES, ESS, Pr, S, Sm = species(z)
    return (S, Sm * E, Sm * ES, ESS), (Sm, ES, ESS, E * Pr)


def evaluate(config, state, *, P=None, a0=None):
    if type(config) is not Config or type(state) not in (list, tuple) or len(state) != 7:
        raise Rejected("fixed Config and seven-component state required")
    z = species(state[:6])
    L = number(state[6], "L", positive=True)
    if L > 4 * config.L0:
        raise Rejected("length outside (0,4L0]")
    P = number(sum(PUBLISHED_INITIAL[:3]) if P is None else P, "P", positive=True)
    a0 = number(sum(PUBLISHED_INITIAL[1:3]) if a0 is None else a0, "a0", nonnegative=True)
    if a0 > P:
        raise Rejected("invalid prepared reference occupancy")
    a = z[1] + z[2]
    extension = (L - config.L0) / config.L0 - config.lam * (a - a0) / P
    Ec = config.K * P * extension**2 / 2
    g = -config.K * config.lam * extension
    Tc = config.K * P * extension / config.L0
    factor = 1.0 if L <= config.L0 else config.phi
    Us = factor * config.k_s * (L - config.L0)**2 / (2 * config.L0)
    Ts = factor * config.k_s * (L - config.L0) / config.L0
    velocity = -(Tc + Ts) / config.gamma
    bp, bm = bare_rates(z)
    if not all(math.isfinite(v) and v > 0 for v in bp + bm):
        raise Rejected("invalid bare rates or underflow")
    affinities = tuple(math.log(p) - math.log(m) - nu * g / THETA
                       for p, m, nu in zip(bp, bm, NU_A))
    try:
        jp = tuple(config.rate_scale * p * math.exp(-nu * g / (2 * THETA))
                   for p, nu in zip(bp, NU_A))
        jm = tuple(config.rate_scale * m * math.exp(nu * g / (2 * THETA))
                   for m, nu in zip(bm, NU_A))
    except OverflowError as error:
        raise Rejected("rate exponential overflow") from error
    if config.rate_scale > 0 and any(v <= 0 for v in jp + jm):
        raise Rejected("rate underflow")
    current = tuple(p - m for p, m in zip(jp, jm))
    dz = tuple(sum(row[i] * current[i] for i in range(4)) for row in STOICH)
    gradient = tuple(math.log(value) + (g if i in (1, 2) else 0)
                     for i, value in enumerate(z)) + (Tc + Ts,)
    ode = dz + (velocity,)
    Dchem = THETA * sum(j * aff for j, aff in zip(current, affinities))
    Dmech = config.gamma * velocity**2
    Gchem = THETA * sum(value * (math.log(value) - 1) for value in z)
    result = {"ode": ode, "gradient": gradient, "bare_forward": bp, "bare_reverse": bm,
              "forward": jp, "reverse": jm, "currents": current, "affinities": affinities,
              "log_ratios": affinities if config.rate_scale > 0 else None,
              "g": g, "Ec": Ec, "Us": Us, "Gchem": Gchem, "G": Gchem + Ec + Us,
              "Dchem": Dchem, "Dmech": Dmech,
              "gradient_dot_ode": sum(v * d for v, d in zip(gradient, ode))}
    finite = ode + gradient + jp + jm + affinities + tuple(result[k] for k in
             ("g", "Ec", "Us", "Gchem", "G", "Dchem", "Dmech", "gradient_dot_ode"))
    if not all(math.isfinite(value) for value in finite) or Dchem < -1e-14 or Dmech < 0:
        raise Rejected("nonfinite state or invalid dissipation")
    return result


def equilibrium_oracle(initial_species=None):
    z = species(PUBLISHED_INITIAL if initial_species is None else initial_species)
    enzyme, substrate = totals(z)
    def constraint(s):
        E = enzyme / (1 + s + s * s)
        return 2 * s + 2 * s * s + E * (s + 2 * s * s) - substrate
    lo, hi = 0.0, max(1.0, substrate)
    for _ in range(100):
        mid = (lo + hi) / 2
        if mid == lo or mid == hi:
            break
        if constraint(mid) < 0:
            lo = mid
        else:
            hi = mid
    s = (lo + hi) / 2
    E = enzyme / (1 + s + s * s)
    eq = (E, s * E, s * s * E, s * s, s, s)
    return {"species": eq, "s_bracket": (lo, hi), "constraint_residual": constraint(s),
            "enzyme_total": enzyme, "substrate_total": substrate}


def _rk4(config, augmented, h, P, a0):
    def rhs(y):
        diagnostic = evaluate(config, y[:7], P=P, a0=a0)
        return diagnostic["ode"] + (diagnostic["Dchem"], diagnostic["Dmech"])
    k1 = rhs(augmented)
    k2 = rhs(tuple(y + h * k / 2 for y, k in zip(augmented, k1)))
    k3 = rhs(tuple(y + h * k / 2 for y, k in zip(augmented, k2)))
    k4 = rhs(tuple(y + h * k for y, k in zip(augmented, k3)))
    return tuple(y + h * (a + 2 * b + 2 * c + d) / 6
                 for y, a, b, c, d in zip(augmented, k1, k2, k3, k4))


def simulate(config, dt, horizon, sample_every=100, initial_species=None):
    if type(config) is not Config:
        raise Rejected("fixed Config required")
    dt = number(dt, "dt", positive=True)
    horizon = number(horizon, "horizon", nonnegative=True)
    if type(sample_every) is not int or sample_every < 1:
        raise Rejected("sample_every must be a positive integer")
    initial = species(PUBLISHED_INITIAL if initial_species is None else initial_species)
    P, substrate = totals(initial)
    a0 = initial[1] + initial[2]
    y = initial + (float(config.L0), 0.0, 0.0)
    initial_diag = evaluate(config, y[:7], P=P, a0=a0)
    G0 = initial_diag["G"]
    t, accepted, rejected = 0.0, 0, 0
    max_material_error = max_defect = maximum_G_increase = peak_Us = 0.0
    max_identity_error = abs(initial_diag["gradient_dot_ode"] + initial_diag["Dchem"] + initial_diag["Dmech"])
    minimum_species = min(initial)
    min_length = max_length = config.L0
    trajectory = []
    def sample():
        d = evaluate(config, y[:7], P=P, a0=a0)
        return {"time": t, "state": y[:7], "G": d["G"], "Gchem": d["Gchem"],
                "Ec": d["Ec"], "Us": d["Us"], "Dchem": d["Dchem"], "Dmech": d["Dmech"],
                "currents": d["currents"], "affinities": d["affinities"],
                "forward": d["forward"], "reverse": d["reverse"],
                "integrated_Dchem": y[7], "integrated_Dmech": y[8],
                "energy_dissipation_defect": d["G"] - G0 + y[7] + y[8],
                "accepted_steps": accepted, "bound_enzyme_amount": y[1] + y[2],
                "remaining_substrate_equivalents": y[4] + y[5] + y[1] + 2 * y[2],
                "product_equivalents": 2 * y[3]}
    trajectory.append(sample())
    while t < horizon:
        if accepted >= config.max_steps:
            raise Rejected("finite accepted-step cap exhausted; no result published")
        h = min(dt, horizon - t)
        old_G = evaluate(config, y[:7], P=P, a0=a0)["G"]
        trial = None
        for halving in range(config.max_halvings + 1):
            try:
                if t + h == t:
                    raise Rejected("time step underflow")
                proposed = _rk4(config, y, h, P, a0)
                d = evaluate(config, proposed[:7], P=P, a0=a0)
                if not all(math.isfinite(v) for v in proposed):
                    raise Rejected("nonfinite augmented state")
                if d["G"] > old_G + config.monotone_tolerance * (1 + abs(old_G)):
                    raise Rejected("free-energy monotonicity guard")
                trial = proposed
                break
            except (Rejected, OverflowError):
                rejected += 1
                h /= 2
        if trial is None:
            raise Rejected("finite halving cap exhausted; no result published")
        # No stage/candidate is clipped or committed until all guards pass.
        y, t = trial, t + h
        accepted += 1
        maximum_G_increase = max(maximum_G_increase, d["G"] - old_G)
        peak_Us = max(peak_Us, d["Us"])
        enzyme_now, substrate_now = totals(y[:6])
        max_material_error = max(max_material_error, abs(enzyme_now - P), abs(substrate_now - substrate))
        minimum_species = min(minimum_species, *y[:6])
        min_length, max_length = min(min_length, y[6]), max(max_length, y[6])
        max_defect = max(max_defect, abs(d["G"] - G0 + y[7] + y[8]))
        max_identity_error = max(max_identity_error, abs(d["gradient_dot_ode"] + d["Dchem"] + d["Dmech"]))
        if accepted % sample_every == 0:
            trajectory.append(sample())
    if trajectory[-1]["time"] != t:
        trajectory.append(sample())
    eq = equilibrium_oracle(initial)["species"]
    final = evaluate(config, y[:7], P=P, a0=a0)
    return {"schema": "digital_cell_computational_chemical_reference_v1", "config": asdict(config),
            "source": "Avanzini_Falasco_Esposito_2020_eq5_7_fig1",
            "initial_condition": "PUBLISHED" if initial_species is None else "EXPLICIT_CONTROL",
            "coupled_extension": "NEW_DESIGNED_REFERENCE_NOT_PUBLISHED_SOURCE_MODEL",
            "theta": THETA, "theta_meaning": "normalized_chemical_energy_per_concentration_not_empirical_target_theta",
            "calibrated": False, "native_runtime_modified": False, "ATP_mapping_established": False,
            "species_order": SPECIES, "P": P, "a0": a0, "initial_state": initial + (config.L0,),
            "final_state": y[:7], "trajectory": trajectory,
            "summary": {"accepted_steps": accepted, "rejected_attempts": rejected, "time": t,
                        "G_initial": G0, "G_final": final["G"], "integrated_Dchem": y[7],
                        "integrated_Dmech": y[8], "final_energy_dissipation_defect": final["G"] - G0 + y[7] + y[8],
                        "maximum_absolute_energy_dissipation_defect": max_defect,
                        "maximum_gradient_identity_error": max_identity_error,
                        "maximum_G_increase": maximum_G_increase, "peak_Us": peak_Us,
                        "maximum_conservation_error": max_material_error, "minimum_species": minimum_species,
                        "length_range": (min_length, max_length), "Ec_final": final["Ec"], "Us_final": final["Us"],
                        "source_equilibrium_distance": max(abs(a - b) for a, b in zip(y[:6], eq)),
                        "remaining_substrate_equivalents": y[4] + y[5] + y[1] + 2 * y[2],
                        "product_equivalents": 2 * y[3], "substrate_total": substrate,
                        "equilibrium_distance_scope": "bare_source_oracle_not_general_coupled_equilibrium",
                        "energy_defect_repaired": False, "free_energy_drop_called_heat": False}}
