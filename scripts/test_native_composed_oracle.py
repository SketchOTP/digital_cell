import math,unittest
from qualify_native_composed import source,NU,near,same
from qualify_chemical_work_owners import elastic,force

class IndependentOracleControls(unittest.TestCase):
    def test_full_source_gradient_identity_and_moieties(self):
        n=[.1,.05,.05,.01,1.,10.];p={'theta':1.,'source_rate_scale':1.};V=.7;g=.31
        f,r,a,j,nd,dc=source(n,V,g,p)
        mu=[math.log(z/V)+(g if i in (1,2) else 0) for i,z in enumerate(n)]
        near(sum(x*y for x,y in zip(mu,nd)),-dc)
        near(sum(nd[:3]),0);near(sum(x*y for x,y in zip(nd,[0,1,2,2,1,1])),0)
        self.assertGreater(dc,0)
        for k in range(4):near(math.log(f[k]/r[k]),a[k])
    def test_zero_source_scale_does_not_invent_flux(self):
        f,r,a,j,nd,dc=source([.1,.05,.05,.01,1.,10.],1.,.2,{'theta':1.,'source_rate_scale':0.})
        self.assertEqual(f+r+j+nd,[0.]*18);self.assertEqual(dc,0);self.assertTrue(any(x!=0 for x in a))
    def test_newborn_primitive_and_floor_counterexample_remain(self):
        near(elastic(20.,0.,2.,.1),0);near(force(20.,0.,2.,.1),0)
        before=elastic(.004,.0005,.0005);after=2*elastic(.002,.00025,.00025)
        near(after-before,-.00153125)
    def test_coherent_shape_and_missing_state_rejected(self):
        expected={'id':1,'n':[.1,.2],'tags':[{'root':'original','fraction':1.}]}
        for altered in [{'id':1,'n':[.1,.2]},dict(expected,id=2),dict(expected,tags=[{'root':'original','fraction':.5}]*2)]:
            with self.assertRaises(ValueError):same(altered,expected)
    def test_nonfinite_numeric_evidence_rejected(self):
        for x in (float('inf'),float('nan')):
            with self.assertRaises(ValueError):near(x,0.)

if __name__=='__main__':unittest.main()
