import copy,json,math,os,pathlib,unittest
from qualify_native_composed import source,NU,near,same,birth,event_check,rejection_controls
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

@unittest.skipUnless(os.environ.get('NATIVE_COMPOSED_INPUT'),'full raw campaign fixture provided by hosted workflow')
class RawCampaignAdversarialControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.raw=json.loads(pathlib.Path(os.environ['NATIVE_COMPOSED_INPUT']).read_text())
        cls.protocol=json.loads((pathlib.Path(__file__).resolve().parents[1]/'docs/dc-m4-native-composed-protocol.json').read_text())
    def test_coherent_birth_tag_split_rejected(self):
        w=copy.deepcopy(self.raw['cases'][0]['coarse']['initial']);old=w['cells'][0]['material_tags'][0][0]
        half={k:(v if k=='root' else v/2) for k,v in old.items()};w['cells'][0]['material_tags'][0]=[half,dict(half)]
        with self.assertRaises(ValueError):birth(w,self.raw['cases'][0]['config'])
    def test_hidden_native_chemistry_rejected(self):
        w=copy.deepcopy(self.raw['cases'][0]['coarse']['initial']);w['cells'][0]['mesh']['interior']['f']=.1
        with self.assertRaises(ValueError):birth(w,self.raw['cases'][0]['config'])
    def test_false_action_extent_and_missing_closure_mapping_rejected(self):
        run=self.raw['cases'][0]['coarse'];p=self.raw['cases'][0]['config']
        growth=copy.deepcopy(run['events'][1]);growth['event']['extent']=.21
        with self.assertRaises(ValueError):event_check(growth,p,self.protocol['schedule'][1])
        fission=copy.deepcopy(run['events'][4]);fission['mapping'].pop()
        with self.assertRaises(ValueError):event_check(fission,p,self.protocol['schedule'][4])
    def test_missing_or_false_negative_evidence_rejected(self):
        altered=dict(self.raw,controls=self.raw['controls'][:-1])
        with self.assertRaises(ValueError):rejection_controls(altered)
        controls=list(self.raw['controls']);controls[0]=dict(controls[0],unchanged=False)
        with self.assertRaises(ValueError):rejection_controls(dict(self.raw,controls=controls))
    def test_extra_serialized_owner_cell_rejected(self):
        controls=list(self.raw['controls']);c=copy.deepcopy(controls[0]);raw=json.loads(c['before_restart']);raw['data']['cells'].append(copy.deepcopy(raw['data']['cells'][0]));c['before_restart']=c['after_restart']=json.dumps(raw);controls[0]=c
        with self.assertRaises(ValueError):rejection_controls(dict(self.raw,controls=controls))
    def test_altered_standalone_control_law_rejected(self):
        for index,key,value in [(4,'k',.5),(7,'gamma',1e-9)]:
            controls=list(self.raw['controls']);c=copy.deepcopy(controls[index]);c['config'][key]=value;controls[index]=c
            with self.assertRaises(ValueError):rejection_controls(dict(self.raw,controls=controls))

if __name__=='__main__':unittest.main()
