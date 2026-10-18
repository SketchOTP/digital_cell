import copy,math,unittest
from calibration_native_checks import recover,holdout_errors,coarse_state_obstruction


def rows(strains=(-.15,-.05,.1,.2),offsets=(-.04,.04),K=3,lam=.4,theta=1.5):
    out=[]
    for strain in strains:
        for da in offsets:
            g=-K*lam*(strain-lam*da/.2);bp=[1,.01,.005,.05];bm=[.1,.05,.05,.001]
            out.append(dict(strain=strain,da=da,P=.2,L0=1,coupling_tension=K*(.2*strain-lam*da),
                            bare_forward=bp,bare_reverse=bm,forward=[p*math.exp(-nu*g/(2*theta)) for p,nu in zip(bp,(0,1,0,-1))],reverse=[m*math.exp(nu*g/(2*theta)) for m,nu in zip(bm,(0,1,0,-1))]))
    return out

class RecoveryTests(unittest.TestCase):
    def test_blind_nondefault_truth_and_holdouts(self):
        fit=recover(rows())
        for k,v in [('K',3),('lambda',.4),('theta',1.5)]:self.assertAlmostEqual(fit[k],v,places=12)
        errors=holdout_errors(rows((-.11,.04,.17),(-.025,.025)),fit)
        self.assertLess(max(errors.values()),1e-12);self.assertIsNone(fit['epsilon_A'])
    def test_no_truth_or_supplied_affinity_consumed(self):
        r=rows()
        for x in r:x['g']=1e99;x['design_truth']={'K':1e99}
        self.assertAlmostEqual(recover(r)['K'],3)
    def test_rank_deficiency_rejected(self):
        r=rows()
        for x in r:x['da']=.2*x['strain']
        with self.assertRaises(ValueError):recover(r)
    def test_missing_excitation_rejected(self):
        r=rows(lam=0)
        with self.assertRaises(ValueError):recover(r)
    def test_bad_directional_rate_rejected(self):
        r=rows();r[0]['forward'][1]=0
        with self.assertRaises(ValueError):recover(r)
    def test_nonfinite_observation_rejected(self):
        r=rows();r[0]['coupling_tension']=float('nan')
        with self.assertRaises(ValueError):recover(r)
    def test_heldout_corruption_detected(self):
        fit=recover(rows());h=rows((-.11,),(.025,));h[0]['coupling_tension']+=.01
        self.assertGreater(holdout_errors(h,fit)['max_force_error'],.009)
    def test_full_coarse_mapping_counterexample(self):
        v=coarse_state_obstruction();self.assertFalse(v['exact_four_parameter_native_coarse_mapping_admissible'])
        self.assertGreater(v['minimax_energy_error_lower_bound'],.2)
        self.assertAlmostEqual(v['active_rate_gap_same_mapped_state'],.045,places=13)
class IndependentModelTests(unittest.TestCase):
    def test_self_consistent_diagnostic_mutant_rejected(self):
        from calibration_native_checks import independent_diagnostics
        from qualify_calibration_native import compare_tree
        import copy,math
        v=[[math.cos(i*math.pi/3),math.sin(i*math.pi/3)] for i in range(6)]
        ref=independent_diagnostics([.1,.05,.05,.01,1.,.1],v,[1.]*6)
        bad=copy.deepcopy(ref);bad['total_g']+=.2;bad['gchem']+=.2
        self.assertGreater(compare_tree(bad,ref),.19)

class ArtifactGuardTests(unittest.TestCase):
    def test_nonfinite_and_short_vectors_rejected(self):
        from qualify_calibration_native import maximum,finite_tree
        for a,b in [([float('nan')],[0.]),([1.],[1.,2.]),([],[])]:
            with self.assertRaises(ValueError):maximum(a,b)
        with self.assertRaises(ValueError):finite_tree({'x':[float('inf')]})
    def test_nan_holdout_rejected(self):
        import copy
        # Reuse valid independently prepared fixture, then corrupt one observation.
        row={'strain':.1,'da':.01,'P':.2,'L0':1.,'coupling_tension':float('nan'),
             'forward':[1.]*4,'reverse':[1.]*4,'bare_forward':[1.]*4,'bare_reverse':[1.]*4}
        with self.assertRaises(ValueError):holdout_errors([row],{'K':2.,'lambda':.75,'theta':1.})
        row['coupling_tension']=0.;row['forward'][1]=float('nan')
        with self.assertRaises(ValueError):holdout_errors([row],{'K':2.,'lambda':.75,'theta':1.})

if __name__=='__main__':unittest.main()
