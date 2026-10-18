import copy,math,unittest
from chemical_body_reference import model,source,birth,remap,area,BIRTH_AREA
class BodyReferenceTests(unittest.TestCase):
    def fixture(self,scale=1.):
        v=birth();rest=[math.hypot(v[(i+1)%24][0]-v[i][0],v[(i+1)%24][1]-v[i][1]) for i in range(24)]
        return source(),[[scale*x for x in p] for p in v],rest
    def test_published_amount_derivative_and_balanced_bath(self):
        n,v,r=self.fixture();d=model(n,v,r,[0]);expected=[.089,.005,-.094,.049,-.9,.985]
        self.assertLess(max(abs(a-b) for a,b in zip(expected,d['amount_derivative'])),1e-13);self.assertLess(max(abs(x) for p in d['osmotic_forces'] for x in p),1e-14)
    def test_volume_scaled_bimolecular_amount_rates(self):
        n,v,r=self.fixture(1.2);d=model(n,v,r,[0],lam=0.)
        self.assertAlmostEqual(d['forward'][1],n[5]*n[0]/d['volume'],places=14)
        self.assertAlmostEqual(d['reverse'][1],n[1],places=14)
    def test_finite_bath_gradient_conjugacy(self):
        n,v,r=self.fixture(1.02);d=model(n,v,r,[0]);h=1e-6
        for i,a in ((0,0),(7,1),(22,0)):
            plus=copy.deepcopy(v);minus=copy.deepcopy(v);plus[i][a]+=h;minus[i][a]-=h
            derivative=(model(n,plus,r,[0])['total_g']-model(n,minus,r,[0])['total_g'])/(2*h)
            self.assertLess(abs(derivative+d['combined_forces'][i][a]),1e-7)
    def test_midpoint_energy_and_gradient_pullback(self):
        n,v,r=self.fixture(1.02);before=model(n,v,r,[0]);n2,v2,r2=remap(n,v,r);after=model(n2,v2,r2,[0,1])
        self.assertAlmostEqual(before['total_g'],after['total_g'],places=12)
        self.assertAlmostEqual(area(v),area(v2),places=12)
        self.assertLess(max(abs(after['combined_forces'][2*i][a]+.5*after['combined_forces'][(2*i-1)%48][a]+.5*after['combined_forces'][2*i+1][a]-before['combined_forces'][i][a]) for i in range(24) for a in range(2)),1e-12)
    def test_omitted_osmotic_force_fails_identity(self):
        n,v,r=self.fixture(1.02);d=model(n,v,r,[0]);self.assertLess(abs(d['gradient_identity_error']),1e-12);self.assertGreater(abs(d['omitted_osmotic_identity_error']),1e-5)
    def test_equilibrium_source_is_stationary(self):
        n,v,r=self.fixture();n=source('ClosedEquilibrium');d=model(n,v,r,[0],'ClosedEquilibrium')
        self.assertLess(max(abs(x) for x in d['amount_derivative']),1e-14)
if __name__=='__main__':unittest.main()
