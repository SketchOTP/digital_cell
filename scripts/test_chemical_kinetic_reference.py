"""Source equations, new coupling and numerical controls; no empirical fitting."""
import dataclasses
import math
import unittest

import chemical_kinetic_reference as ref


def coupled(**changes):
    values = dict(K=2.0, lam=0.75, k_s=1.0, L0=1.0, gamma=1.0, phi=0.7)
    values.update(changes)
    return ref.Config(**values)


class ReferenceTests(unittest.TestCase):
    def test_published_initial_rates_and_derivative(self):
        diagnostic = ref.evaluate(ref.Config(), ref.PUBLISHED_INITIAL + (1.0,))
        for actual, expected in zip(diagnostic["currents"], (0.9, -0.04, -0.045, 0.049)):
            self.assertAlmostEqual(actual, expected, places=14)
        for actual, expected in zip(diagnostic["ode"], (0.089, 0.005, -0.094, 0.049, -0.9, 0.985, 0.0)):
            self.assertAlmostEqual(actual, expected, places=14)
        enzyme, substrate = ref.totals(ref.PUBLISHED_INITIAL)
        self.assertAlmostEqual(enzyme, 0.2)
        self.assertAlmostEqual(substrate, 1.27)

    def test_exact_stoichiometric_conservation_for_arbitrary_currents(self):
        for currents in ((1, -2, 3, -4), (0.9, -0.04, -0.045, 0.049), (-3, 2, -7, 1)):
            dz = [sum(row[i] * currents[i] for i in range(4)) for row in ref.STOICH]
            self.assertAlmostEqual(sum(dz[:3]), 0.0)
            self.assertAlmostEqual(dz[4] + dz[5] + dz[1] + 2 * dz[2] + 2 * dz[3], 0.0)
            self.assertAlmostEqual(dz[4] + dz[5] + dz[1] + 2 * dz[2], -2 * currents[3])

    def test_equilibrium_oracle_independent_polynomial(self):
        oracle = ref.equilibrium_oracle()
        E, ES, ESS, Pr, S, Sm = oracle["species"]
        polynomial = 200 * S**4 + 400 * S**3 + 313 * S**2 + 93 * S - 127
        self.assertLess(abs(polynomial), 1e-12)
        self.assertAlmostEqual(S, Sm)
        self.assertAlmostEqual(Pr, S**2)
        self.assertAlmostEqual(ES, S * E)
        self.assertAlmostEqual(ESS, S**2 * E)
        diag = ref.evaluate(ref.Config(), oracle["species"] + (1.0,))
        self.assertLess(max(abs(j) for j in diag["currents"]), 1e-15)
        run = ref.simulate(coupled(), 0.02, 1.0, initial_species=oracle["species"])
        self.assertLess(max(abs(a - b) for a, b in zip(run["final_state"], oracle["species"] + (1.0,))), 1e-13)

    def test_gradient_matches_independent_energy_differences(self):
        cfg = coupled()
        state = ref.PUBLISHED_INITIAL + (1.2,)
        diag = ref.evaluate(cfg, state)
        h = 1e-6
        for index in range(7):
            plus, minus = list(state), list(state)
            plus[index] += h
            minus[index] -= h
            derivative = (ref.evaluate(cfg, plus)["G"] - ref.evaluate(cfg, minus)["G"]) / (2 * h)
            self.assertAlmostEqual(derivative, diag["gradient"][index], delta=2e-8)

    def test_reciprocal_mechanical_and_conversion_cross_derivatives(self):
        cfg, h = coupled(), 1e-6
        state = list(ref.PUBLISHED_INITIAL + (1.2,))
        plus, minus = state.copy(), state.copy()
        plus[1] += h
        minus[1] -= h
        cross_T = (ref.evaluate(cfg, plus)["gradient"][-1]
                   - ref.evaluate(cfg, minus)["gradient"][-1]) / (2 * h)
        plus, minus = state.copy(), state.copy()
        plus[-1] += h
        minus[-1] -= h
        cross_g = (ref.evaluate(cfg, plus)["g"] - ref.evaluate(cfg, minus)["g"]) / (2 * h)
        self.assertAlmostEqual(cross_T, -cfg.K * cfg.lam / cfg.L0, delta=1e-9)
        self.assertAlmostEqual(cross_T, cross_g, delta=1e-9)

    def test_directional_bias_signs_and_elementary_local_detailed_balance(self):
        cfg = coupled()
        state = ref.PUBLISHED_INITIAL + (1.2,)
        diag = ref.evaluate(cfg, state)
        for i, nu in enumerate((0, 1, 0, -1)):
            expected = math.log(diag["bare_forward"][i] / diag["bare_reverse"][i]) - nu * diag["g"]
            actual = math.log(diag["forward"][i] / diag["reverse"][i])
            self.assertAlmostEqual(actual, expected, places=13)
            delta_G = sum(diag["gradient"][row] * ref.STOICH[row][i] for row in range(6))
            self.assertAlmostEqual(delta_G, -actual, places=13)
            self.assertGreaterEqual(diag["currents"][i] * actual, -1e-14)
        self.assertLess(diag["forward"][3] / diag["bare_forward"][3], 1.0)
        self.assertGreater(diag["forward"][1] / diag["bare_forward"][1], 1.0)

    def test_gradient_dot_ode_is_negative_complete_dissipation(self):
        for cfg in (ref.Config(), coupled(), coupled(rate_scale=2.0), coupled(rate_scale=0.0)):
            for L in (0.8, 1.0, 1.2):
                diag = ref.evaluate(cfg, ref.PUBLISHED_INITIAL + (L,))
                self.assertAlmostEqual(diag["gradient_dot_ode"],
                                       -diag["Dchem"] - diag["Dmech"], delta=2e-13)
                self.assertGreaterEqual(diag["Dchem"], -1e-14)

    def test_simulated_conservation_monotonicity_and_unrepaired_defect(self):
        run = ref.simulate(coupled(), 0.02, 3.0, sample_every=10)
        summary = run["summary"]
        self.assertLess(summary["maximum_conservation_error"], 1e-12)
        self.assertGreater(summary["minimum_species"], 0)
        self.assertGreater(summary["integrated_Dchem"], 0)
        self.assertGreater(summary["integrated_Dmech"], 0)
        previous = run["trajectory"][0]["G"]
        for row in run["trajectory"]:
            self.assertLessEqual(row["G"], previous + 1e-12)
            previous = row["G"]
            self.assertAlmostEqual(row["energy_dissipation_defect"], row["G"] - summary["G_initial"]
                                   + row["integrated_Dchem"] + row["integrated_Dmech"], places=14)
        self.assertLessEqual(summary["maximum_G_increase"], 1e-12)
        self.assertGreaterEqual(summary["peak_Us"], max(row["Us"] for row in run["trajectory"]))
        for row in run["trajectory"]:
            self.assertIn("Gchem", row)
            self.assertEqual(len(row["currents"]), 4)
            self.assertEqual(len(row["affinities"]), 4)
        self.assertFalse(summary["energy_defect_repaired"])
        self.assertFalse(summary["free_energy_drop_called_heat"])
        self.assertFalse(run["calibrated"])
        self.assertAlmostEqual(summary["remaining_substrate_equivalents"]
                               + summary["product_equivalents"], summary["substrate_total"], places=12)

    def test_refinement_reduces_defect_and_state_difference(self):
        runs = [ref.simulate(coupled(), h, 2.0, sample_every=1000) for h in (0.04, 0.02, 0.01)]
        defects = [abs(r["summary"]["final_energy_dissipation_defect"]) for r in runs]
        self.assertLess(defects[1], defects[0])
        self.assertLess(defects[2], defects[1])
        coarse_change = max(abs(a - b) for a, b in zip(runs[0]["final_state"], runs[1]["final_state"]))
        fine_change = max(abs(a - b) for a, b in zip(runs[1]["final_state"], runs[2]["final_state"]))
        self.assertLess(fine_change, coarse_change / 5)

    def test_zero_controls_and_time_rescaling(self):
        bare = ref.simulate(ref.Config(), 0.02, 1.0)
        zero_lambda = ref.simulate(coupled(lam=0), 0.02, 1.0)
        self.assertEqual(bare["final_state"], zero_lambda["final_state"])
        zero_rates = ref.simulate(coupled(rate_scale=0), 0.02, 1.0)
        self.assertEqual(zero_rates["initial_state"], zero_rates["final_state"])
        slow = ref.simulate(coupled(), 0.02, 1.0)
        fast = ref.simulate(coupled(rate_scale=2, gamma=0.5), 0.01, 0.5)
        self.assertLess(max(abs(a - b) for a, b in zip(slow["final_state"], fast["final_state"])), 1e-13)

    def test_invalid_inputs_finite_caps_and_no_input_mutation(self):
        for changes in (dict(K=-1), dict(lam=-1), dict(phi=2), dict(gamma=0), dict(rate_scale=-1),
                        dict(K=float("nan")), dict(L0=float("inf")), dict(K=True), dict(max_steps=True)):
            with self.assertRaises(ref.Rejected):
                ref.Config(**changes)
        cfg, state = coupled(max_steps=1), list(ref.PUBLISHED_INITIAL + (1.0,))
        original = state.copy()
        with self.assertRaises(ref.Rejected):
            ref.simulate(cfg, 0.02, 1.0)
        for invalid in (state[:6] + [0.0], [0.0] + state[1:], state[:6] + [5.0]):
            with self.assertRaises(ref.Rejected):
                ref.evaluate(cfg, invalid)
        with self.assertRaises(ref.Rejected):
            ref.simulate(cfg, 0, 1)
        with self.assertRaisesRegex(ref.Rejected, "halving cap"):
            ref.simulate(coupled(K=1e300, max_halvings=1), 0.02, 1.0)
        with self.assertRaises(dataclasses.FrozenInstanceError):
            cfg.K = 7
        self.assertEqual(state, original)


if __name__ == "__main__":
    unittest.main()
