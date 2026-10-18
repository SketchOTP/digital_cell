"""Synthetic metadata controls; no fixture is a real calibration observation."""
import copy
import unittest

import calibration_source_admission as admission


def fixture(targets=None):
    targets = list(admission.PARAMETERS) if targets is None else targets
    return {
        "id": "SYNTHETIC_SCHEMA_CONTROL_ONLY",
        "kind": "EXTERNAL_EXPERIMENTAL",
        "targets": targets,
        "evidence_ids": ["SYNTHETIC_PROOF_REFERENCE"],
        "independence": "VERIFIED",
        "target_gates": {
            parameter: {
                gate: {"status": "VERIFIED", "evidence_ids": ["SYNTHETIC_PROOF_REFERENCE"],
                       "rationale": "Synthetic schema assertion for validation controls only."}
                for gate in admission.GATES
            } for parameter in targets
        },
    }


class AdmissionTests(unittest.TestCase):
    def test_missing_source_or_mapping_is_not_admitted(self):
        candidate = fixture()
        candidate["independence"] = "MISSING"
        result = admission.evaluate(candidate)
        self.assertEqual(set(result["per_parameter"].values()), {"NOT_ADMITTED_MISSING_EVIDENCE"})
        candidate["independence"] = "VERIFIED"
        for parameter in admission.PARAMETERS:
            candidate["target_gates"][parameter]["unit_species_mapping"] = {
                "status": "MISSING", "evidence_ids": [],
                "rationale": "No independently established A/W/P and force/length map."}
        result = admission.evaluate(candidate)
        self.assertEqual(set(result["per_parameter"].values()), {"NOT_ADMITTED_MISSING_EVIDENCE"})
        self.assertFalse(result["admitted"])

    def test_circular_source_cannot_override_with_verified_metadata(self):
        for kind in ("PROTOTYPE", "INTERNAL_GENERATED"):
            candidate = fixture()
            candidate["kind"] = kind
            result = admission.evaluate(candidate)
            self.assertEqual(set(result["per_parameter"].values()), {"REJECTED_NONINDEPENDENT"})
            self.assertFalse(result["admitted"])
        candidate = fixture()
        candidate["independence"] = "REJECTED"
        self.assertEqual(admission.evaluate(candidate)["per_parameter"]["K"], "REJECTED_NONINDEPENDENT")

    def test_other_domains_and_native_constants_are_reference_only(self):
        for kind in ("NATIVE_CONSTITUTIVE", "OTHER_DOMAIN"):
            candidate = fixture()
            candidate["kind"] = kind
            result = admission.evaluate(candidate)
            self.assertEqual(set(result["per_parameter"].values()), {"NOT_ADMITTED_INCOMPATIBLE"})
            self.assertEqual(result["reference_disposition"], "REFERENCE_ONLY")
            self.assertFalse(result["calibrated"])

    def test_unknown_proof_reference_is_defect_even_if_missing(self):
        candidate = fixture(["K"])
        gate = candidate["target_gates"]["K"]["observations"]
        for status in ("VERIFIED", "MISSING", "INCOMPATIBLE"):
            gate["status"] = status
            gate["evidence_ids"] = ["NONEXISTENT_REFERENCE"]
            with self.assertRaisesRegex(ValueError, "EVIDENCE_DEFECT"):
                admission.evaluate(candidate)

    def test_verified_requires_refs_and_meaningful_rationale(self):
        for refs, rationale in (([], "An apparently adequate declaration of evidence."),
                                (["SYNTHETIC_PROOF_REFERENCE"], ""),
                                (["SYNTHETIC_PROOF_REFERENCE"], "   "),
                                (["SYNTHETIC_PROOF_REFERENCE"], "yes")):
            candidate = fixture(["theta"])
            gate = candidate["target_gates"]["theta"]["observations"]
            gate.update(evidence_ids=refs, rationale=rationale)
            with self.assertRaisesRegex(ValueError, "EVIDENCE_DEFECT"):
                admission.evaluate(candidate)

    def test_typo_bool_and_wrong_types_do_not_coerce(self):
        mutations = (
            lambda c: c.update(independence=True),
            lambda c: c.update(kind=False),
            lambda c: c.update(targets="K"),
            lambda c: c.update(evidence_ids=("SYNTHETIC_PROOF_REFERENCE",)),
            lambda c: c.update(id=123),
            lambda c: c["target_gates"]["K"]["observations"].update(status="VERIFED"),
            lambda c: c["target_gates"]["K"]["observations"].update(status=True),
            lambda c: c["target_gates"]["K"]["observations"].update(rationale=True),
            lambda c: c["target_gates"]["K"]["observations"].update(evidence_ids=[False]),
        )
        for mutate in mutations:
            candidate = fixture()
            mutate(candidate)
            with self.assertRaisesRegex(ValueError, "EVIDENCE_DEFECT"):
                admission.evaluate(candidate)

    def test_unknown_stale_or_omitted_fields_are_defects(self):
        mutations = (
            lambda c: c.update(admitted=True),
            lambda c: c.update(schema="stale_schema"),
            lambda c: c.pop("independence"),
            lambda c: c["target_gates"]["K"].pop("observations"),
            lambda c: c["target_gates"]["K"]["observations"].update(pass_gate=True),
            lambda c: c["target_gates"].update(unknown={}),
            lambda c: c["targets"].append("unknown"),
            lambda c: c["targets"].append("K"),
        )
        for mutate in mutations:
            candidate = fixture()
            mutate(candidate)
            with self.assertRaisesRegex(ValueError, "EVIDENCE_DEFECT"):
                admission.evaluate(candidate)

    def test_target_subset_and_incompatible_priority(self):
        candidate = fixture(["K", "lambda"])
        candidate["target_gates"]["K"]["variable_mapping"]["status"] = "INCOMPATIBLE"
        candidate["target_gates"]["K"]["observations"]["status"] = "MISSING"
        result = admission.evaluate(candidate)
        self.assertEqual(result["per_parameter"], {
            "K": "NOT_ADMITTED_INCOMPATIBLE", "lambda": "ELIGIBLE_FOR_REVIEW",
            "theta": "NOT_TARGETED", "epsilon_A": "NOT_TARGETED"})
        self.assertEqual(result["gate_findings"]["K"]["incompatible_gates"], ["variable_mapping"])

    def test_empty_evidence_and_targets_remain_nonadmission(self):
        candidate = fixture(["epsilon_A"])
        candidate["evidence_ids"] = []
        candidate["independence"] = "MISSING"
        for gate in candidate["target_gates"]["epsilon_A"].values():
            gate.update(status="MISSING", evidence_ids=[], rationale="No source evidence acquired.")
        self.assertEqual(admission.evaluate(candidate)["per_parameter"]["epsilon_A"],
                         "NOT_ADMITTED_MISSING_EVIDENCE")
        candidate = fixture([])
        self.assertEqual(set(admission.evaluate(candidate)["per_parameter"].values()), {"NOT_TARGETED"})

    def test_all_complete_synthetic_fixture_is_only_eligible(self):
        for kind in ("EXTERNAL_EXPERIMENTAL", "EXTERNAL_COMPUTATIONAL"):
            candidate = fixture()
            candidate["kind"] = kind
            snapshot = copy.deepcopy(candidate)
            result = admission.evaluate(candidate)
            self.assertEqual(candidate, snapshot)
            self.assertEqual(set(result["per_parameter"].values()), {"ELIGIBLE_FOR_REVIEW"})
            self.assertIs(result["admitted"], False)
            self.assertIs(result["calibrated"], False)
            self.assertIs(result["semantic_review_required"], True)
            self.assertIs(result["eligibility_is_proof_authentication"], False)
            self.assertEqual(result, admission.evaluate(candidate))


if __name__ == "__main__":
    unittest.main()
