"""Independent analytic and Decimal controls; all numerical inputs illustrative."""
from decimal import Decimal, localcontext
from fractions import Fraction as F
import unittest

import reference_mapping as rm


def dec(value):
    return Decimal(value.numerator) / Decimal(value.denominator)


class MappingTests(unittest.TestCase):
    def test_nonunit_similarity_from_energy_differences(self):
        s = rm.Scales(F(2), F(3), F(5), F(7), F(11))
        old = (F(3), F(2, 5), F(2), F(3), F(4), F(1))
        new = old[:4] + (F(5), F(3, 2))
        keys = ("K", "lambda", "P", "L0", "L", "a")
        lifted_old = tuple(s.to_source(k, v) for k, v in zip(keys, old))
        lifted_new = tuple(s.to_source(k, v) for k, v in zip(keys, new))
        self.assertEqual(rm.coupling(*lifted_new) - rm.coupling(*lifted_old),
                         F(11) * (rm.coupling(*new) - rm.coupling(*old)))
        self.assertEqual(s.to_source("gamma", F(4)), F(33))
        self.assertEqual(s.to_source("epsilon_A", F(2)), F(22, 7))
        self.assertEqual(s.to_source("kf", F(2)), F(8, 75))
        self.assertEqual(s.to_source("c0", F(3)), F(21, 5))

    def test_dimensionless_groups_not_identified_by_units(self):
        s = rm.Scales(F(2), F(3), F(5), F(7), F(11))
        K, theta, epsA, c0, P, ks, L = F(3), F(2), F(5), F(1, 2), F(7), F(4), F(6)
        source = {k: s.to_source(k, v) for k, v in
                  (("K", K), ("theta", theta), ("epsilon_A", epsA), ("c0", c0),
                   ("P", P), ("ks", ks), ("L", L))}
        self.assertEqual(source["K"] / source["theta"], K / theta)
        self.assertEqual(source["K"] / (source["epsilon_A"] * source["c0"]), K / (epsA * c0))
        self.assertEqual(source["K"] * source["P"] / (source["ks"] * source["L"]), K * P / (ks * L))

    def test_two_port_energy_scale_and_independent_inverse(self):
        fixture = rm.TwoPortFixture(F(5), F(3, 2), F(2, 3), F(7, 3), F(2), F(3))
        self.assertEqual(fixture.K, F(1215, 56))
        self.assertEqual(fixture.lam, F(8, 27))
        self.assertEqual(fixture.energy(F(4), F(1)), F(125, 72))
        target = rm.coupling(fixture.K, fixture.lam, F(2), F(3), F(4), F(1))
        self.assertEqual(target, F(125, 168))
        self.assertNotEqual(target, fixture.energy(F(4), F(1)))

    def test_two_port_jacobian_by_independent_finite_differences(self):
        fixture = rm.TwoPortFixture(F(5), F(3, 2), F(2, 3), F(7, 3), F(2), F(3))
        L, a, h = F(4), F(1), F(1, 100)
        T = (fixture.energy(L + h, a) - fixture.energy(L - h, a)) / (2 * h * fixture.e)
        g = (fixture.energy(L, a + h) - fixture.energy(L, a - h)) / (2 * h * fixture.e)
        self.assertEqual(fixture.target_efforts(L, a), (T, g))
        left, right = fixture.discrete_work(F(2), F(1, 4), F(4), F(3, 2))
        self.assertEqual(left + right, (fixture.energy(F(4), F(3, 2))
                                      - fixture.energy(F(2), F(1, 4))) / fixture.e)
        self.assertNotEqual(right, 0)  # The second port cannot be discarded.

    def test_rotary_signed_work_and_drag_power(self):
        angle, torque, radius, drag = F(-2, 3), F(5), F(7), F(3)
        q, force, linear_drag = rm.rotary_chart(angle, torque, drag, radius)
        self.assertEqual(q * force, F(-10, 3))
        angular_velocity = F(-4, 5)
        self.assertEqual(linear_drag * (radius * angular_velocity)**2,
                         drag * angular_velocity**2)

    def test_binary_mixture_force_agrees_but_affinity_does_not(self):
        N, k, d, q, x, h = F(3), F(2), F(4), F(5), F(1, 4), F(1, 100)
        plus = rm.binary_mixture(N, k, d, q + h, x)
        minus = rm.binary_mixture(N, k, d, q - h, x)
        self.assertEqual((plus[0] - minus[0]) / (2 * h), (plus[1] - minus[1]) / (2 * h))
        up = rm.binary_mixture(N, k, d, q, x + h)
        down = rm.binary_mixture(N, k, d, q, x - h)
        actual_g = (up[0] - down[0]) / (2 * h)
        shifted_g = (up[1] - down[1]) / (2 * h)
        self.assertEqual(actual_g - shifted_g, N * k * d**2 * (1 - 2 * x) / 2)
        self.assertNotEqual(actual_g, shifted_g)

    def test_midpoint_contact_does_not_remove_finite_mismatch(self):
        start = rm.binary_mixture(F(3), F(2), F(4), F(5), F(1, 2))
        end = rm.binary_mixture(F(3), F(2), F(4), F(5), F(3, 4))
        self.assertNotEqual(end[0] - start[0], end[1] - start[1])
        symmetric = rm.binary_mixture(F(3), F(2), F(4), F(5), F(1, 4))
        self.assertEqual(symmetric[2], end[2])
        self.assertNotEqual(start[2], end[2])  # Symmetric endpoints hide path change.

    def test_finite_fuel_exact_derivative_obstructs_affine_battery(self):
        fuel = rm.FiniteFuel(F(10), F(3), F(4), F(2))
        self.assertEqual(fuel.derivative(F(0)), F(-41, 30))
        for xi in (F(0), F(1), F(2), F(9)):
            self.assertLess(fuel.derivative(xi), 0)
        # Positive coordinate/energy scales cannot turn a strictly negative
        # second derivative of source energy into the zero curvature of EA.
        self.assertLess(F(7)**2 / F(11) * fuel.derivative(F(1)), 0)

    def test_local_bound_against_independent_log_energy(self):
        fuel, X = rm.FiniteFuel(F(10), F(3), F(4), F(2)), F(2)
        bound = fuel.local_bounds(X)
        with localcontext() as context:
            context.prec = 65
            nT, nD, nP, temp, x = map(dec, (fuel.nT, fuel.nD, fuel.nP, fuel.kBT, X))
            affinity_change = temp * (((nT - x) * nD * nP / (nT * (nD + x) * (nP + x))).ln())
            primitive = lambda y: y * y.ln() - y
            integrated_change = temp * (
                primitive(nT) - primitive(nT - x) - x * nT.ln()
                - primitive(nD + x) + primitive(nD) + x * nD.ln()
                - primitive(nP + x) + primitive(nP) + x * nP.ln())
            self.assertLess(abs(affinity_change), dec(bound["affinity_error_bound"]))
            self.assertLess(abs(integrated_change), dec(bound["integrated_energy_error_bound"]))
            self.assertLess(affinity_change, 0)
            self.assertLess(integrated_change, 0)

    def test_invalid_scales_types_and_boundaries_reject(self):
        for value in (F(0), F(-1), 1, True, 1.0, float("nan"), float("inf")):
            with self.assertRaises(ValueError):
                rm.Scales(value, F(1), F(1), F(1), F(1))
            with self.assertRaises(ValueError):
                rm.TwoPortFixture(F(1), F(1), F(1), value, F(1), F(1))
        fuel = rm.FiniteFuel(F(10), F(3), F(4), F(2))
        for X in (F(0), F(-1), F(10), F(11), 1.0):
            with self.assertRaises(ValueError):
                fuel.local_bounds(X)
        with self.assertRaises(ValueError):
            rm.coupling(F(1), F(1), F(1), F(1), F(5), F(0))
        with self.assertRaises(ValueError):
            rm.binary_mixture(F(1), F(1), F(1), F(0), F(2))
        with self.assertRaises(ValueError):
            rm.rotary_chart(F(0), F(1), F(1), F(0))

    def test_summary_is_deterministic_and_does_not_admit_calibration(self):
        result = rm.evaluate_reference_checks()
        self.assertEqual(result, rm.evaluate_reference_checks())
        self.assertEqual(result["total_cases"], 25)
        self.assertFalse(result["full_calibration_established"])
        self.assertFalse(result["source_species_mapping_established"])
        self.assertFalse(result["fixture_built_or_measured"])
        self.assertEqual(result["admitted_target_observations"], 0)


if __name__ == "__main__":
    unittest.main()
