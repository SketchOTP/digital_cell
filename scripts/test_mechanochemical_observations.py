import io,math,unittest
from mechanochemical_observations import path_work,independent_work,window_work,group_rows,selected_pairs

class WorkTests(unittest.TestCase):
    def test_linear_force_and_reverse(self):
        q=[(i/100,2*i/100+3,0) for i in range(101)]
        self.assertAlmostEqual(path_work(q),4.);self.assertAlmostEqual(independent_work(q),4.)
        self.assertAlmostEqual(path_work(q[::-1]),-4.)
    def test_rectangle_orientation(self):
        q=[(0,1,0),(2,1,1),(2,3,2),(0,3,3),(0,1,4)]
        self.assertAlmostEqual(path_work(q),-4.);self.assertAlmostEqual(independent_work(q),-4.)
    def test_clipping_backtracking(self):
        q=[(-1,2,0),(2,2,1),(0,2,2),(3,2,3)]
        self.assertAlmostEqual(window_work(q,.5,1.5),2.)
        self.assertAlmostEqual(window_work(q[::-1],.5,1.5),-2.)
    def test_bad_source_width_and_status(self):
        header=b'A_dist-Y B_dist-Y Y_force time(sec) Status\n'
        for row in (b'1 2 3 4\n',b'1 2 3 4 1000.5\n',b'1 2 nan 4 1000\n'):
            with self.assertRaises(ValueError):group_rows(io.BytesIO(header+row))
    def test_complete_pair_boundaries(self):
        groups=[{'status':s,'points':[(0,0,0)]*200} for s in (40,1020,1021,40,1040,1041)]
        pairs,count=selected_pairs(groups,200,3);self.assertEqual(count,1);self.assertEqual(pairs[0][1]['status'],1020)
    def test_missing_intervals_are_not_integrated(self):
        q=[(0,2,0,False),(1,2,1,False),(10,2,10,True),(11,2,11,False)]
        self.assertAlmostEqual(path_work(q),4.);self.assertAlmostEqual(independent_work(q),4.)
        self.assertAlmostEqual(window_work(q,-1,12),4.)
        self.assertAlmostEqual(path_work([p[:3] for p in q]),22.)
    def test_known_instrument_markers_are_recorded(self):
        data=b'A_dist-Y B_dist-Y Y_force time(sec) Status\n1 1 -2 0 1000\n#Skipped\n2 2 -2 1 1000\n#End Write'
        _,g,n,_,markers=group_rows(io.BytesIO(data));self.assertEqual(n,2);self.assertTrue(g[0]['points'][1][3]);self.assertEqual(markers,{'skipped':1,'end_write':1,'equal_timestamps':0,'reversed_timestamps':0})
    def test_terminal_marker_rejects_appended_run(self):
        data=b'A_dist-Y B_dist-Y Y_force time(sec) Status\n1 1 -2 0 1000\n#End Write\n2 2 -2 1 1000\n'
        with self.assertRaises(ValueError):group_rows(io.BytesIO(data))

if __name__=='__main__':unittest.main()
