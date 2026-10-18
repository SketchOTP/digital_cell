import math
import unittest
from dataclasses import replace
import chemical_work_ownership as m


class OwnershipTests(unittest.TestCase):
    def setUp(self):
        self.world = m.prepared_world()

    def transact(self, action):
        return m.commit(self.world,m.propose(self.world,action))

    def test_native_primitive_matches_independent_integral_and_force(self):
        p = m.Params()
        # Simpson integration of the independently stated native force law.
        for h,y,L in ((0.,1.,2.),(.0005,.001,.02),(.01,.002,.2),(1.,.2,.4),(1.,.2,8.)):
            r=max(h/p.rho,1e-15)
            def f(x):
                raw=p.k_s*(x-r)/max(r,x/4,.001)
                return raw if x < r else raw*(h/(h+y) if h+y else 0)
            # Split at the analytic kink before independent quadrature.
            boundaries=sorted(set((r,L,max(r,.001)*4)))
            lo,hi=min(r,L),max(r,L)
            cuts=[lo]+[v for v in boundaries if lo<v<hi]+[hi]
            integral=0.
            for a,b in zip(cuts,cuts[1:]):
                N=2000
                step=(b-a)/N
                integral+=step/3*(f(a)+f(b)+sum((4 if i%2 else 2)*f(a+i*step) for i in range(1,N)))
            if L<r:
                integral=-integral
            U,F=m.elastic(L,h,y,p)
            self.assertAlmostEqual(U,integral,places=8)
            self.assertAlmostEqual(F,f(L),places=12)

    def test_gradient_and_zero_mature_continuity(self):
        for h,y,L in ((0.,1.,.5),(.0005,.001,.002),(.01,.002,.03),(1.,.2,.4),(1.,.2,8.)):
            step=min(L*1e-5,1e-6)
            up=m.elastic(L+step,h,y,self.world.params)[0]
            dn=m.elastic(L-step,h,y,self.world.params)[0]
            self.assertAlmostEqual((up-dn)/(2*step),m.elastic(L,h,y,self.world.params)[1],places=7)
        self.assertLess(m.elastic(1.,1e-12,1.,self.world.params)[0],1e-10)

    def test_growth_has_fixed_stoichiometry_before_energy(self):
        action={"kind":"growth","cell":"cell:0","edge":0,"extent":.2}
        target,_,record=self.transact(action)
        self.assertAlmostEqual(target.cells[0].n[5],9.8)
        self.assertAlmostEqual(target.cells[0].n[4],1.02)
        self.assertAlmostEqual(target.cells[0].edges[0].young,.28)
        self.assertEqual(record["action"],action)
        self.assertAlmostEqual(m.invariants(target)["substrate"],m.invariants(self.world)["substrate"])

    def test_maturation_conserves_edge_mass_with_actual_storage_change(self):
        target,_,record=self.transact({"kind":"maturation","cell":"cell:0","edge":0,"extent":.04})
        before=self.world.cells[0].edges[0]
        after=target.cells[0].edges[0]
        self.assertAlmostEqual(before.young+before.mature,after.young+after.mature)
        self.assertAlmostEqual(after.mature-before.mature,.04)
        self.assertGreater(record["energy_changes"]["native_elastic"],0)

    def test_finite_transport_debits_outside_and_allows_uphill_realization(self):
        target,_,record=self.transact({"kind":"transport","cell":"cell:0","species":"Sm","direction":"in","amount":.2})
        self.assertAlmostEqual(target.outside_sm,9.8)
        self.assertAlmostEqual(target.cells[0].n[5],10.2)
        self.assertGreater(record["pair"]["delta_g"],0)
        self.assertLess(record["chemical_free_energy_release"],0)

    def test_logistic_pair_detailed_balance_and_ensemble_entropy(self):
        for dg in (-12.,-1.,0.,1.,12.):
            rates=m.pair_rates(dg,self.world.params)
            self.assertAlmostEqual(math.log(rates["forward"]/rates["reverse"]),-dg)
            self.assertAlmostEqual(rates["forward"]+rates["reverse"],1.)
            self.assertGreaterEqual(rates["ensemble_pair_entropy"],0)

    def test_tiny_attempt_rate_cannot_erase_reverse_channel(self):
        self.assertGreater(math.exp(-100.),0.)
        with self.assertRaises(m.Rejected):
            m.pair_rates(100.,m.Params(kappa=1e-320))

    def test_fission_partitions_all_sources_and_adds_exact_closure_material(self):
        target,receipt,record=self.transact({"kind":"fission","cell":"cell:0","i":0,"j":3})
        self.assertEqual(len(target.cells),2)
        old=self.world.cells[0]
        neck=math.dist(old.vertices[0],old.vertices[3])
        for d in target.cells:
            self.assertAlmostEqual(d.P,old.P/2)
            self.assertAlmostEqual(d.a0,old.a0/2)
            self.assertAlmostEqual(d.R_ref,old.R_ref/2)
            self.assertEqual(d.edges[-1].mature,0)
            self.assertAlmostEqual(d.edges[-1].young,neck*self.world.params.rho)
        self.assertEqual(target.outside_s,self.world.outside_s)
        self.assertAlmostEqual(sum(m.area(c.vertices) for c in target.cells),m.area(old.vertices))
        self.assertEqual(m.digest(m.reverse(target,receipt),True),m.digest(self.world,True))
        self.assertAlmostEqual(sum(record["energy_changes"].values()),record["pair"]["delta_g"])

    def test_receipts_recover_physics_but_are_stale_after_inverse(self):
        action={"kind":"growth","cell":"cell:0","edge":0,"extent":.2}
        target,receipt,_=self.transact(action)
        recovered=m.reverse(target,receipt)
        self.assertEqual(m.digest(recovered,True),m.digest(self.world,True))
        self.assertGreater(recovered.revision,target.revision)
        with self.assertRaises(m.Rejected):
            m.reverse(recovered,receipt)
        with self.assertRaises(m.Rejected):
            m.reverse(replace(target,owner="foreign"),receipt)
        with self.assertRaises(m.Rejected):
            m.commit(target,m.propose(self.world,action))

    def test_remesh_neutral_only_when_absolute_reference_floor_inactive(self):
        loaded,_,_=self.transact({"kind":"maturation","cell":"cell:0","edge":0,"extent":.04})
        event={"kind":"remesh","cell":"cell:0","edge":0}
        target,_,record=m.commit(loaded,m.propose(loaded,event))
        self.assertGreater(m.energy(loaded)["cells"][0]["elastic"],0)
        self.assertAlmostEqual(record["pair"]["delta_g"],0.,places=12)
        self.assertEqual(len(target.cells[0].edges),7)
        old=m.elastic(.004,.0005,.0005,self.world.params)[0]
        new=2*m.elastic(.002,.00025,.00025,self.world.params)[0]
        self.assertGreater(abs(old-new),1e-4)

    def test_no_donor_clipping_and_complete_rollback(self):
        saved=m.digest(self.world)
        for action in ({"kind":"growth","cell":"cell:0","edge":0,"extent":100.},
                       {"kind":"maturation","cell":"cell:0","edge":0,"extent":10.},
                       {"kind":"transport","cell":"cell:0","species":"S","direction":"in","amount":100.}):
            with self.assertRaises(m.Rejected):
                m.propose(self.world,action)
            self.assertEqual(saved,m.digest(self.world))

    def test_bad_types_owners_parameters_and_stoichiometry_rejected(self):
        for value in (True,float("nan"),float("inf"),"1"):
            with self.assertRaises(m.Rejected):
                m.Params(theta=value)
        with self.assertRaises(m.Rejected):
            m.propose(self.world,{"kind":"growth","cell":"cell:0","edge":False,"extent":.2})
        with self.assertRaises(m.Rejected):
            m.propose(self.world,{"kind":"growth","cell":"cell:0","edge":0,"extent":.2,"free_energy_credit":10})
        with self.assertRaises(m.Rejected):
            m.propose(self.world,{"kind":"growth","cell":"missing","edge":0,"extent":.2})

    def test_lost_fraction_and_duplicated_material_origin_rejected(self):
        c=self.world.cells[0]
        e=c.edges[0]
        lost=replace(e,tags=(replace(e.tags[0],fraction=.5),))
        duplicate=replace(e,tags=e.tags+e.tags)
        for corrupted in (lost,duplicate):
            bad_cell=replace(c,edges=(corrupted,)+c.edges[1:])
            with self.assertRaises(m.Rejected):
                m.validate(replace(self.world,cells=(bad_cell,)))
        # A lawful midpoint split retains both halves of the same root.
        action={"kind":"remesh","cell":"cell:0","edge":0}
        target,_,_=self.transact(action)
        self.assertEqual(sum(t.fraction for e in target.cells[0].edges
                             for t in e.tags if t.root==c.edges[0].tags[0].root),1.)

    def test_drive_controls_distinguish_chemical_from_mechanical_energy(self):
        action={"kind":"maturation","cell":"cell:0","edge":0,"extent":.04}
        deltas=[]
        for mu in (-50.,50.):
            w=m.prepared_world(replace(self.world.params,mu_m=mu))
            _,_,record=m.commit(w,m.propose(w,action))
            deltas.append(record["pair"]["delta_g"])
            self.assertAlmostEqual(record["mechanical_storage_change"]-record["chemical_free_energy_release"],record["pair"]["delta_g"])
        self.assertLess(deltas[0],0);self.assertGreater(deltas[1],0)


if __name__ == "__main__":
    unittest.main()
