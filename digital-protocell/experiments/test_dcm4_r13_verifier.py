"""Focused control fixtures for R13 fail-closed numerical attribution."""
import unittest
import dcm4_r13_intra_operator_causal_decomposition_verify as v

class R13VerifierControls(unittest.TestCase):
    def test_native_order_and_isolated_material_energy(self):
        active=[0.2,0.8,0.1,0.3]; inactive=[0.6,0.1,0.4,0.5]; lengths=[0.8,1.2,0.7,1.1]
        full=v.kernel(active,inactive,lengths,"FULL",10)
        same=v.kernel(active,inactive,lengths,"TRANSPORT_THEN_REACTION",10)
        self.assertEqual(full,same)
        for path in v.PATHS:
            a,b,cost,_=v.kernel(active,inactive,lengths,path,10)
            self.assertAlmostEqual(sum(a+b),sum(active+inactive),places=12)
            self.assertGreaterEqual(min(a+b),0)
            if path=="TRANSPORT_ONLY": self.assertEqual(cost,0)
        self.assertNotEqual(full[:2],v.kernel(active,inactive,lengths,"REACTION_THEN_TRANSPORT",10)[:2])
    def test_insufficient_energy_rejected(self):
        with self.assertRaises(AssertionError): v.kernel([0.1]*4,[0.5]*4,[1]*4,"FULL",0)
    def test_unique_dominance_and_cancellation_controls(self):
        vectors={"FULL":[1.,0.],"REACTION_ONLY":[0.9,0.],"TRANSPORT_ONLY":[0.08,0.],"INTERACTION":[0.02,0.]}
        errors={k:1e-6 for k in vectors}; phases={k:-1. for k in vectors}
        candidate,_=v.attribution(vectors,phases,errors,errors)
        self.assertEqual(candidate,"REACTION_ONLY")
        vectors.update(REACTION_ONLY=[3.,0.],TRANSPORT_ONLY=[-2.,0.],INTERACTION=[0.,0.])
        candidate,_=v.attribution(vectors,phases,errors,errors)
        self.assertIsNone(candidate)
    def test_phase_agreement_required(self):
        vectors={"FULL":[1.,0.],"REACTION_ONLY":[1.,0.],"TRANSPORT_ONLY":[0.,0.],"INTERACTION":[0.,0.]}
        errors={k:1e-6 for k in vectors}; phases={k:1. for k in vectors}
        self.assertIsNone(v.attribution(vectors,phases,errors,errors)[0])
    def test_fail_closed_coverage(self):
        records=[{"nomination":"REACTION_ONLY"}]*23+[{"nomination":None}]*7
        self.assertEqual(v.classify(records)[0],"G_TO_P_INTERNAL_ATTRIBUTION_INCONCLUSIVE")
        records[23]={"nomination":"REACTION_ONLY"}
        self.assertEqual(v.classify(records)[0],v.CLASSES[0])
    def test_boundary_payload_and_label_corruption_rejected(self):
        import json
        serialized=json.dumps({"amount":1.0},separators=(",",":"))
        fnv=0xcbf29ce484222325
        for byte in serialized.encode(): fnv=((fnv^byte)*0x100000001b3)&((1<<64)-1)
        digest=f"fnv1a64:{fnv:016x}"
        snapshot={"input_boundary_digest":digest,"input_boundary_serialized":serialized,"input_boundary":{"amount":1.0}}
        v.bind_boundary(snapshot,digest)
        snapshot["input_boundary"]={"amount":2.0}
        with self.assertRaises(AssertionError): v.bind_boundary(snapshot,digest)
        snapshot["input_boundary"]={"amount":1.0}
        with self.assertRaises(AssertionError): v.bind_boundary(snapshot,"fnv1a64:0000000000000000")
    def test_duplicate_arm_rejected(self):
        arms=[{"arm":i,"snapshots":[{"checkpoint_step":s} for s in [3694,7389,11083]]} for i in range(1,11)]
        v.bind_arms(arms,arms)
        duplicate=arms[:-1]+[arms[0]]
        with self.assertRaises(AssertionError): v.bind_arms(duplicate,arms)
    def test_old_r8_symmetry_status_and_error_corruption_rejected(self):
        import copy
        scale={"signs":[{"status":"VALID","active_delta":[1.0],"inactive_delta":[-1.0]},{"status":"VALID","active_delta":[-1.0],"inactive_delta":[1.0]}],"sign_symmetry":{"status":"PASS","opposite_error":0.0}}
        reference={"scales":[copy.deepcopy(scale),copy.deepcopy(scale)],"status":"VALID"}
        self.assertTrue(v.r8_mode_valid(reference))
        corrupted=copy.deepcopy(reference); corrupted["scales"][0]["sign_symmetry"]["status"]="FAIL"
        with self.assertRaises(AssertionError): v.r8_mode_valid(corrupted)
        corrupted=copy.deepcopy(reference); corrupted["scales"][0]["sign_symmetry"]["opposite_error"]=0.1
        with self.assertRaises(AssertionError): v.r8_mode_valid(corrupted)
        corrupted=copy.deepcopy(reference); corrupted["status"]="G_NONLINEAR_OR_NONSMOOTH"
        with self.assertRaises(AssertionError): v.r8_mode_valid(corrupted)
    def test_nonfinite_rejected(self):
        with self.assertRaises(AssertionError): v.close(float("nan"),0)

if __name__=="__main__": unittest.main()
