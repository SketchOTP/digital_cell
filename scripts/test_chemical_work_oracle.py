import copy,json,math,unittest
from qualify_chemical_work_owners import force,elastic,world,prepared,same,qualify,event
import chemical_work_ownership as candidate
from dataclasses import replace

class OracleTests(unittest.TestCase):
    def test_quadrature_known_spring_and_newborn(self):
        self.assertAlmostEqual(elastic(2,1,0),.5,places=11)
        self.assertAlmostEqual(elastic(.5,1,0),.125,places=11)
        self.assertEqual(elastic(8,0,1),0)
        self.assertAlmostEqual(force(8,1,0),3.5,places=12)
    def test_unowned_input_cannot_replace_preparation(self):
        p={'theta':1.,'k_s':1.,'rho':1.,'K':.25,'lam':.1,'mu_y':-.25,'mu_m':-.5,'yield_y':.9,'kappa':1.,'total_volume':10.};w=prepared(p);world(w);wrong=copy.deepcopy(w);wrong['cells'][0]['edges'][0]['tags'][0]['young']=.05
        with self.assertRaises(ValueError):same(wrong,w)
    def test_impossible_origin_fraction_rejected(self):
        p={'theta':1.,'k_s':1.,'rho':1.,'K':.25,'lam':.1,'mu_y':-.25,'mu_m':-.5,'yield_y':.9,'kappa':1.,'total_volume':10.};w=prepared(p);w['cells'][0]['edges'][0]['tags'][0]['fraction']=.5
        with self.assertRaises(ValueError):world(w)
    def test_coherent_alternative_extent_rejected_by_protocol(self):
        v=candidate.fixture_campaign();w=candidate.prepared_world();_,_,r=candidate.commit(w,candidate.propose(w,{'kind':'growth','cell':'cell:0','edge':0,'extent':.21}));v['events'][1]=r
        with self.assertRaisesRegex(ValueError,'sealed exact event inventory'):qualify(v)
    def test_coherent_alternative_drive_and_grid_rejected(self):
        v=candidate.fixture_campaign();w=candidate.prepared_world(replace(candidate.Params(),mu_m=-49.));_,_,r=candidate.commit(w,candidate.propose(w,{'kind':'maturation','cell':'cell:0','edge':0,'extent':.04}));v['drive_controls'][0]=r
        with self.assertRaisesRegex(ValueError,'sealed chemical drive'):qualify(v)
        v=candidate.fixture_campaign();x=v['force_grid'][0];x.update(L=.003,mature=.0003,young=.0002);x['energy'],x['force']=candidate.elastic(x['L'],x['mature'],x['young'],candidate.Params())
        with self.assertRaisesRegex(ValueError,'sealed reference force'):qualify(v)
    def test_forward_remesh_tag_shape_and_unaffected_edges_rejected(self):
        v=json.loads(json.dumps(candidate.fixture_campaign()));r=v['events'][4];e=r['after']['cells'][0]['edges'][0];t=e['tags'][0];half={k:(value/2 if k!='root' else value) for k,value in t.items()};e['tags']=[dict(half),dict(half)]
        with self.assertRaises(ValueError):event(r,False)
        v=json.loads(json.dumps(candidate.fixture_campaign()));r=v['events'][4];r['after']['cells'][0]['edges'][-1]['id']='unowned-edge-id'
        with self.assertRaises(ValueError):event(r,False)

if __name__=='__main__':unittest.main()
