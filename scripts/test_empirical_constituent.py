import copy,math,unittest
from empirical_constituent import fit_tau,normalized_wave,prediction_rmse,independent_minimum_check
SEL={'samples_per_period':16000,'sample_rate_hz':200000,'step_sample':4000,'baseline_slice':[1000,3000],'plateau_slice':[6000,10000],'fit_slice':[4010,4800],'relaxation_tau_seconds_range':[1e-5,.01]}
class EmpiricalTests(unittest.TestCase):
    def wave(self,tau):return [0. if i<4000 else 1-math.exp(-(i-4000)/200000/tau) for i in range(16000)]
    def test_recovers_nondefault_physical_relaxation(self):
        w=self.wave(.000731);f=fit_tau([w,w],SEL);self.assertAlmostEqual(f['tau_seconds'],.000731,places=10);self.assertLess(prediction_rmse(w,f['tau_seconds'],SEL),1e-8);self.assertLess(independent_minimum_check([w,w],SEL,f['tau_seconds'])['relative_optimizer_difference'],1e-6)
    def test_fixed_prediction_detects_heldout_different_rate(self):
        f=fit_tau([self.wave(.0004)],SEL);self.assertGreater(prediction_rmse(self.wave(.002),f['tau_seconds'],SEL),.1)
    def test_flat_and_boundary_rejected(self):
        for w in ([.5]*16000,self.wave(.1)):
            with self.assertRaises(ValueError):fit_tau([w],SEL)
    def test_nonfinite_fit_rejected(self):
        w=self.wave(.001);w[4100]=float('nan')
        with self.assertRaises(ValueError):fit_tau([w],SEL)
    def test_projection_shape_is_gain_and_offset_invariant(self):
        u=self.wave(.0004);s=[2.]*16000;x=[3.-v for v in u];y=[7.-2*v for v in u]
        a,_=normalized_wave([x,y,s],SEL,10.);b,_=normalized_wave([x,y,s],SEL,1.)
        self.assertLess(max(abs(v-w) for v,w in zip(a,b)),1e-12)
    def test_zero_denominator_and_amplitude_rejected(self):
        with self.assertRaises(ValueError):normalized_wave([[1.]*16000,[2.]*16000,[0.]*16000],SEL,10.)
        with self.assertRaises(ValueError):normalized_wave([[1.]*16000,[2.]*16000,[1.]*16000],SEL,10.)
if __name__=='__main__':unittest.main()
