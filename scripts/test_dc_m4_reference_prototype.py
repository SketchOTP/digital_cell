"""Substantive reference controls; local algebra fixtures are not run entry."""
import dataclasses
import json
from pathlib import Path
import tempfile
from unittest.mock import patch
from fractions import Fraction as F
import unittest

import dc_m4_reference_prototype as ref


def params(**changes):
    values = dict(K=F(2), lam=F(1, 3), theta=F(1), epsilon_A=F(3),
                  k_s=F(1), gamma=F(1), c0=F(1, 2), h=F(1, 10))
    values.update(changes)
    return ref.Params(**values)


def material(**changes):
    values = dict(P=F(1), M=F(1), rho_s=F(1), phi=F(1, 2))
    values.update(changes)
    return ref.Material(**values)


def accuracy(**changes):
    values = dict(force=F(1, 10**9), width=F(1, 10**10),
                  step_energy=F(1, 10**10), cumulative_energy=F(1, 10**8),
                  max_iterations=80)
    values.update(changes)
    return ref.Accuracy(**values)


def tension_oracle(p, m, a, old):
    """Independent exact linear solve when both endpoints are on tensile branch."""
    drag = p.gamma / p.h
    kc = p.K * m.P / m.L0**2
    ks = m.phi * p.k_s / m.L0
    preferred = m.L0 * (1 + p.lam * a / m.P)
    return (drag * old + kc * (preferred - old / 2)
            + ks * (m.L0 - old / 2)) / (drag + (kc + ks) / 2)


class PrototypeTests(unittest.TestCase):
    def cold(self, **kwargs):
        return ref.Run.cold(params(), material(), accuracy(**kwargs), F(10))

    def assert_whole(self, run):
        p, m, s = run.params, run.material, run.state
        total = (p.epsilon_A * s.A + ref.coupling(p, m, s.a, s.L)
                 + ref.stretch(p, m, s.L) + s.Q_chem + s.Q_drag
                 - p.epsilon_A * run.initial_A)
        self.assertEqual(total, s.signed_energy_error)
        self.assertLessEqual(abs(total), s.absolute_energy_error)
        self.assertLessEqual(s.absolute_energy_error, run.accuracy.cumulative_energy)
        self.assertEqual(s.a + s.b, m.P)
        self.assertEqual(s.A + s.W, run.initial_A + run.initial_W)
        self.assertGreaterEqual(s.Q_chem, 0)
        self.assertGreaterEqual(s.Q_drag, 0)

    def test_cold_and_multiple_reachable_cycles(self):
        run = self.cold()
        self.assertEqual(run.state.a, 0)
        self.assertEqual(run.state.L, run.material.L0)
        self.assertEqual(ref.coupling(run.params, run.material, F(0), run.state.L), 0)
        old_runs = [run]
        for d in (F(1, 4), F(1, 4), F(-1, 8), F(-1, 4), F(1, 8)):
            previous = run
            run = run.cycle(d, (F(1, 10), F(4)))
            self.assertIsNot(run, previous)
            self.assertEqual(run.state.cycles, previous.state.cycles + 1)
            self.assertLessEqual(run.state.A, previous.state.A)
            self.assertGreaterEqual(run.state.Q_chem, previous.state.Q_chem)
            self.assertGreaterEqual(run.state.Q_drag, previous.state.Q_drag)
            self.assert_whole(run)
            old_runs.append(run)
        self.assertEqual(old_runs[0].state.cycles, 0)
        self.assertEqual(len(run.certificates), 5)
        self.assertGreater(run.state.Q_chem, 0)
        self.assertGreater(run.state.Q_drag, 0)

    def test_exact_oracle_and_certified_approximation(self):
        run = self.cold()
        expected = tension_oracle(run.params, run.material, F(1, 4), run.state.L)
        next_run = run.cycle(F(1, 4), (F(1), F(2)))
        c = next_run.certificates[-1]
        self.assertLessEqual(c.lo, expected)
        self.assertGreaterEqual(c.hi, expected)
        self.assertLessEqual(abs(c.root - expected), c.width)
        self.assertLessEqual(c.f_lo, 0)
        self.assertGreaterEqual(c.f_hi, 0)
        self.assertLessEqual(abs(c.force_residual), run.accuracy.force)
        self.assertLessEqual(abs(c.energy_error), run.accuracy.step_energy)
        self.assertEqual(c.energy_error, c.force_residual * (c.root - run.state.L))
        exact = run.cycle(F(1, 4), (expected, expected))
        self.assertEqual(exact.state.L, expected)
        self.assertEqual(exact.certificates[-1].width, 0)
        self.assertEqual(exact.state.absolute_energy_error, 0)

    def test_local_true_rest_crossing_secant(self):
        # Local witness only: cold prototype does not import either preload.
        p, m = params(), material()
        old, new = F(1, 2), F(3, 2)
        secant = ref.stretch_secant(p, m, old, new)
        self.assertEqual(secant, F(-1, 16))
        self.assertEqual(secant * (new - old),
                         ref.stretch(p, m, new) - ref.stretch(p, m, old))
        self.assertNotEqual(secant, p.k_s * ((new + old) / 2 - m.L0) / m.L0)
        self.assertEqual(ref.stretch_secant(p, m, m.L0, m.L0), 0)
        self.assertEqual(ref.stretch_secant(p, m, new, old), secant)

    def test_local_equal_energy_path_still_spends(self):
        # Local identity at preloaded geometry, never a Run constructor.
        p, m = params(lam=F(1)), material()
        a, d, length = F(1, 4), F(1, 2), F(3, 2)
        up, down = ref.reaction_variations(p, m, a, length, d)
        self.assertEqual(up, down)
        self.assertGreater(up, 0)
        z = p.c0 * abs(d) + up / p.epsilon_A
        q = p.epsilon_A * p.c0 * abs(d) + down
        delta = ref.coupling(p, m, a + d, length) - ref.coupling(p, m, a, length)
        self.assertEqual(-p.epsilon_A * z + delta + q, 0)
        self.assertGreater(z, p.c0 * abs(d))

    def test_split_reaction_path_variation_identity(self):
        p, m = params(lam=F(1)), material()
        a, mid, end, length = F(1, 4), F(1, 2), F(3, 4), F(3, 2)
        whole = ref.reaction_variations(p, m, a, length, end - a)
        left = ref.reaction_variations(p, m, a, length, mid - a)
        right = ref.reaction_variations(p, m, mid, length, end - mid)
        self.assertEqual(whole, tuple(x + y for x, y in zip(left, right)))
        # Two alternative cold-reachable runs. Different mechanics staging need
        # not give identical final geometry; both must retain complete ledgers.
        run = self.cold()
        one = run.cycle(F(1, 2), (F(1, 10), F(4)))
        two = run.cycle(F(1, 4), (F(1, 10), F(4))).cycle(F(1, 4), (F(1, 10), F(4)))
        self.assertEqual(one.state.a, two.state.a)
        self.assert_whole(one)
        self.assert_whole(two)

    def test_donor_and_A_exhaustion(self):
        run = self.cold()
        for d in (F(-1, 4), F(5, 4)):
            with self.assertRaises(ref.Rejected):
                run.cycle(d, (F(1, 10), F(4)))
        empty = ref.Run.cold(params(), material(), accuracy(), F(0))
        with self.assertRaisesRegex(ref.Rejected, "pre-stroke"):
            empty.cycle(F(1, 4), (F(1, 10), F(4)))
        unchanged = empty.cycle(F(0), (F(1), F(1)))
        self.assertEqual(unchanged.state.A, 0)
        self.assertEqual(unchanged.state.W, 0)

    def test_late_iteration_failure_preserves_nonzero_sinks(self):
        run = self.cold(max_iterations=1)
        root = tension_oracle(run.params, run.material, F(1, 4), F(1))
        paid = run.cycle(F(1, 4), (root, root))
        snapshot = ref.jsonable(paid)
        self.assertGreater(paid.state.Q_chem, 0)
        self.assertGreater(paid.state.Q_drag, 0)
        with self.assertRaisesRegex(ref.Rejected, "iteration cap"):
            paid.cycle(F(1, 4), (F(1, 10), F(4)))
        self.assertEqual(ref.jsonable(paid), snapshot)

    def test_cumulative_budget_exhaustion_is_atomic(self):
        run = self.cold(cumulative_energy=F(0))
        root = tension_oracle(run.params, run.material, F(1, 4), F(1))
        paid = run.cycle(F(1, 4), (root, root))
        snapshot = ref.jsonable(paid)
        with self.assertRaisesRegex(ref.Rejected, "cumulative"):
            paid.cycle(F(1, 4), (F(1, 10), F(4)))
        self.assertEqual(ref.jsonable(paid), snapshot)
        self.assert_whole(paid)

    def test_late_bracket_failure_preserves_existing_error_and_sinks(self):
        run = self.cold().cycle(F(1, 4), (F(1, 10), F(4)))
        snapshot = ref.jsonable(run)
        self.assertGreater(run.state.absolute_energy_error, 0)
        for bracket in ((F(2), F(3)), (F(0), F(4)), (F(1), F(5)), (F(2), F(1))):
            with self.assertRaises(ref.Rejected):
                run.cycle(F(1, 4), bracket)
            self.assertEqual(ref.jsonable(run), snapshot)

    def test_immutability_and_only_cold_entry(self):
        run = self.cold()
        for target, name, value in ((run, "params", params(K=F(4))),
                                    (run.state, "A", F(100)),
                                    (run.params, "K", F(4)),
                                    (run.material, "P", F(2))):
            with self.assertRaises(dataclasses.FrozenInstanceError):
                setattr(target, name, value)
        with self.assertRaises(ref.Rejected):
            ref.Run(state=run.state)
        with self.assertRaises(ref.Rejected):
            dataclasses.replace(run, params=params(K=F(4)))
        for operation in ("OFF", "rupture", "remesh", "synthesis", "diffusion", "pressure", "change_params"):
            with self.assertRaises(ref.Rejected):
                run.unsupported(operation)

    def test_exact_input_and_domain_guards(self):
        for bad in (1, True, 1.0, float("nan"), float("inf"), "1/2", None):
            with self.assertRaises(ref.Rejected):
                params(K=bad)
            with self.assertRaises(ref.Rejected):
                self.cold().cycle(bad, (F(1), F(2)))
            with self.assertRaises(ref.Rejected):
                ref.Run.cold(params(), material(), accuracy(), bad)
        for change in (dict(K=F(0)), dict(lam=F(-1)), dict(h=F(0)), dict(epsilon_A=F(-1))):
            with self.assertRaises(ref.Rejected):
                params(**change)
        for change in (dict(P=F(0)), dict(M=F(0)), dict(rho_s=F(1, 10**16)),
                       dict(M=F(1, 10000)), dict(phi=F(2))):
            with self.assertRaises(ref.Rejected):
                material(**change)
        for change in (dict(force=F(0)), dict(cumulative_energy=F(-1)),
                       dict(max_iterations=True), dict(max_iterations=0)):
            with self.assertRaises(ref.Rejected):
                accuracy(**change)
        with self.assertRaises(ref.Rejected):
            self.cold().cycle(F(1, 4), (F(1), 2.0))

    def test_cli_exclusive_output(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "demo.json"
            with patch("sys.argv", ["dc_m4_reference_prototype.py", "--output", str(target)]):
                ref.main()
            original = target.read_bytes()
            data = json.loads(original)
            self.assertEqual(data["classification"], "ILLUSTRATIVE_BOUNDED_REFERENCE_ONLY")
            self.assertFalse(data["calibrated"])
            self.assertEqual(data["history"][0]["state"]["a"], "0")
            with patch("sys.argv", ["dc_m4_reference_prototype.py", "--output", str(target)]):
                with self.assertRaises(FileExistsError):
                    ref.main()
            self.assertEqual(target.read_bytes(), original)

    def test_no_repair_and_deterministic_serialization(self):
        payload = ref.demo()
        second = ref.demo()
        self.assertEqual(ref.jsonable(payload), ref.jsonable(second))
        run = payload["history"][-1]
        self.assertEqual(run.state.absolute_energy_error,
                         sum((abs(c.energy_error) for c in run.certificates), F(0)))
        self.assertEqual(run.state.signed_energy_error,
                         sum((c.energy_error for c in run.certificates), F(0)))
        self.assert_whole(run)
        self.assertEqual(ref.jsonable(F(2, 3)), "2/3")


if __name__ == "__main__":
    unittest.main()
