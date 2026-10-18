"""Designed finite-event ownership reference, not identified biology or native fusion.

Event-pair rates describe ensemble reciprocity. Individual uphill transitions
remain legal; no energy decrease is relabelled heat and no ledger is repaired.
"""
from dataclasses import dataclass, replace, asdict
import hashlib
import json
import math


class Rejected(ValueError):
    pass


def finite(x, *, positive=False, nonnegative=False):
    if type(x) not in (int, float) or not math.isfinite(x):
        raise Rejected("finite real scalar required")
    if positive and x <= 0 or nonnegative and x < 0:
        raise Rejected("scalar outside physical domain")
    return float(x)


@dataclass(frozen=True)
class Params:
    theta: float = 1.0
    k_s: float = 1.0
    rho: float = 1.0
    K: float = 0.25
    lam: float = 0.1
    mu_y: float = -0.25
    mu_m: float = -0.5
    yield_y: float = 0.9
    kappa: float = 1.0
    total_volume: float = 10.0

    def __post_init__(self):
        for key, value in asdict(self).items():
            finite(value)
        if min(self.theta, self.k_s, self.rho, self.kappa, self.total_volume) <= 0 or self.rho < 1e-15:
            raise Rejected("positive thermodynamic/constitutive scales required")
        if self.K < 0 or self.lam < 0 or not 0 < self.yield_y <= 1:
            raise Rejected("invalid coupling/yield")


@dataclass(frozen=True)
class Tag:
    root: str
    fraction: float
    young: float
    mature: float


@dataclass(frozen=True)
class Edge:
    id: str
    tags: tuple

    @property
    def young(self):
        return sum(t.young for t in self.tags)

    @property
    def mature(self):
        return sum(t.mature for t in self.tags)


@dataclass(frozen=True)
class Cell:
    id: str
    vertices: tuple
    edges: tuple
    n: tuple
    P: float
    a0: float
    R_ref: float


@dataclass(frozen=True)
class World:
    owner: str
    params: Params
    root_area: float
    cells: tuple
    outside_s: float
    outside_sm: float
    outside_inert: float
    revision: int = 0


def area(vertices):
    return sum(a[0]*b[1]-b[0]*a[1] for a, b in zip(vertices, vertices[1:]+vertices[:1])) / 2


def lengths(cell):
    return tuple(math.hypot(b[0]-a[0], b[1]-a[1])
                 for a, b in zip(cell.vertices, cell.vertices[1:]+cell.vertices[:1]))


def rest(edge, p):
    return max(edge.mature / max(p.rho, 1e-15), 1e-15)


def elastic(length, mature, young, p):
    """Exact primitive of native regularized V4 stretch, zero at L=r."""
    L = finite(length, positive=True)
    h = finite(mature, nonnegative=True)
    y = finite(young, nonnegative=True)
    r = max(h / max(p.rho, 1e-15), 1e-15)
    R = max(r, 0.001)
    phi = h / (h+y) if h+y > 0 else 0.0
    factor = 1.0 if L < r else phi
    if L <= 4*R:
        primitive = (L-r)**2 / (2*R)
    else:
        primitive = (4*R-r)**2/(2*R) + 4*(L-4*R) - 4*r*math.log(L/(4*R))
    force = p.k_s * (L-r) / max(r, L/4, 0.001) * factor
    return p.k_s * primitive * factor, force


def _simple(vertices):
    def cross(a, b, c):
        return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    def on(a, b, c):
        return abs(cross(a,b,c)) <= 1e-12 and all(min(a[k],b[k])-1e-12 <= c[k] <= max(a[k],b[k])+1e-12 for k in (0,1))
    for i, a in enumerate(vertices):
        b = vertices[(i+1)%len(vertices)]
        for j in range(i+1, len(vertices)):
            if j == i+1 or i == 0 and j == len(vertices)-1:
                continue
            c, d = vertices[j], vertices[(j+1)%len(vertices)]
            if (cross(a,b,c)*cross(a,b,d)<0 and cross(c,d,a)*cross(c,d,b)<0
                    or on(a,b,c) or on(a,b,d) or on(c,d,a) or on(c,d,b)):
                return False
    return True


def validate(w):
    if type(w) is not World or type(w.params) is not Params or not w.owner or type(w.owner) is not str:
        raise Rejected("typed owned world required")
    if type(w.cells) is not tuple or not w.cells or type(w.revision) is not int or w.revision < 0:
        raise Rejected("immutable cells/revision required")
    finite(w.root_area, positive=True)
    for value in (w.outside_s, w.outside_sm, w.outside_inert):
        finite(value, nonnegative=True)
    ids = set()
    edge_ids = set()
    origin_fractions = {}
    for c in w.cells:
        if type(c) is not Cell or type(c.vertices) is not tuple or type(c.edges) is not tuple or type(c.n) is not tuple:
            raise Rejected("immutable typed cell required")
        if type(c.id) is not str or not c.id or c.id in ids:
            raise Rejected("duplicate cell owner")
        ids.add(c.id)
        if len(c.vertices) < 3 or len(c.vertices) != len(c.edges) or len(c.n) != 6:
            raise Rejected("cell geometry/source dimensions")
        for v in c.vertices:
            if type(v) is not tuple or len(v) != 2:
                raise Rejected("immutable coordinates required")
            for x in v:
                finite(x)
        if area(c.vertices) <= 1e-12 or min(lengths(c)) <= 1e-12 or len(set(c.vertices)) != len(c.vertices) or not _simple(c.vertices):
            raise Rejected("non-simple/degenerate/negative-area cell")
        for x in c.n:
            finite(x, nonnegative=True)
        finite(c.P, positive=True)
        finite(c.a0, nonnegative=True)
        finite(c.R_ref, positive=True)
        if c.a0 > c.P or abs(sum(c.n[:3])-c.P) > 1e-10:
            raise Rejected("source enzyme/preparation ownership")
        for e in c.edges:
            if type(e) is not Edge or type(e.id) is not str or not e.id or e.id in edge_ids or type(e.tags) is not tuple or not e.tags:
                raise Rejected("edge ownership/ancestry")
            edge_ids.add(e.id)
            for t in e.tags:
                if type(t) is not Tag or type(t.root) is not str or not t.root:
                    raise Rejected("typed material origin required")
                finite(t.fraction, positive=True)
                if t.fraction > 1:
                    raise Rejected("invalid material fraction")
                origin_fractions[t.root] = origin_fractions.get(t.root,0.0)+t.fraction
                finite(t.young, nonnegative=True)
                finite(t.mature, nonnegative=True)
    if any(abs(fraction-1.0) > 1e-12 for fraction in origin_fractions.values()):
        raise Rejected("lost/duplicated ancestral material interval")
    if sum(area(c.vertices)/w.root_area for c in w.cells) >= w.params.total_volume:
        raise Rejected("finite outside volume exhausted")
    return w


def _ideal(n, V, theta):
    return theta * sum(x*(math.log(x/V)-1) if x > 0 else 0.0 for x in n)


def energy(w):
    validate(w)
    p = w.params
    rows = []
    for c in w.cells:
        V = area(c.vertices)/w.root_area
        ls = lengths(c)
        rs = tuple(rest(e,p) for e in c.edges)
        x = (sum(ls)-sum(rs))/c.R_ref-p.lam*(c.n[1]+c.n[2]-c.a0)/c.P
        rows.append({"id":c.id,"volume":V,"g_source":_ideal(c.n,V,p.theta),
                     "g_bound":sum(p.mu_y*e.young+p.mu_m*e.mature for e in c.edges),
                     "elastic":sum(elastic(L,e.mature,e.young,p)[0] for L,e in zip(ls,c.edges)),
                     "coupling":p.K*c.P*x*x/2,"lengths":ls,"rest_lengths":rs})
    Vout = p.total_volume-sum(r["volume"] for r in rows)
    outside = _ideal((w.outside_s,w.outside_sm,w.outside_inert),Vout,p.theta)
    total = outside+sum(r["g_source"]+r["g_bound"]+r["elastic"]+r["coupling"] for r in rows)
    if not math.isfinite(total):
        raise Rejected("nonfinite owned energy")
    return {"total":total,"outside":outside,"outside_volume":Vout,"cells":rows}


def invariants(w):
    validate(w)
    return {"enzyme":sum(sum(c.n[:3]) for c in w.cells),
            "substrate":w.outside_s+w.outside_sm+sum(c.n[1]+2*c.n[2]+2*c.n[3]+c.n[4]+c.n[5]
                       +sum(e.young+e.mature for e in c.edges) for c in w.cells),
            "a0":sum(c.a0 for c in w.cells),"R_ref":sum(c.R_ref for c in w.cells)}


def digest(w, physical=False):
    value = asdict(w)
    if physical:
        value.pop("revision")
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(",",":"),allow_nan=False).encode()).hexdigest()


def pair_rates(delta_g, p):
    x = finite(delta_g)/p.theta
    # Stable logistic, with exact log-ratio reported independently of underflow.
    z = math.exp(-abs(x))
    if z == 0:
        raise Rejected("event-pair rate underflow; no zeroed reverse channel")
    low, high = p.kappa*z/(1+z), p.kappa/(1+z)
    kf, kr = (low,high) if x >= 0 else (high,low)
    if not all(math.isfinite(rate) and rate > 0 for rate in (kf,kr)):
        raise Rejected("computed event-pair rate overflow/underflow")
    return {"forward":kf,"reverse":kr,"log_ratio":-x,
            "reference_occupancies":[0.5,0.5],"ensemble_forward_flux":0.5*kf,"ensemble_reverse_flux":0.5*kr,
            "ensemble_pair_entropy":0.5*(kf-kr)*(-x),"delta_g":delta_g,
            "entropy_meaning":"dimensionless equal-occupancy two-state ensemble witness, not realized-path heat"}


def _index(w, key):
    matches = [i for i,c in enumerate(w.cells) if c.id == key]
    if len(matches) != 1:
        raise Rejected("unknown cell owner")
    return matches[0]


def _change_y(e, change, label):
    if change >= 0:
        return replace(e,tags=e.tags+(Tag(label,1.,change,0.),))
    if -change > e.young:
        raise Rejected("young material donor exhausted")
    fraction = (e.young+change)/e.young
    return replace(e,tags=tuple(replace(t,young=t.young*fraction) for t in e.tags))


def _event(w, action):
    """The event's stoichiometry is fixed before evaluating its energy."""
    validate(w)
    if type(action) is not dict or type(action.get("kind")) is not str:
        raise Rejected("typed event specification required")
    kind = action["kind"]
    fields = {"transport":{"kind","cell","species","direction","amount"},
              "growth":{"kind","cell","edge","extent"},"maturation":{"kind","cell","edge","extent"},
              "fission":{"kind","cell","i","j"},"remesh":{"kind","cell","edge"}}
    if kind not in fields or set(action) != fields[kind]:
        raise Rejected("unknown/stale event fields")
    idx = _index(w,action["cell"])
    c = w.cells[idx]
    n, edges = list(c.n), list(c.edges)
    cells = list(w.cells)
    p = w.params
    label = f"{w.owner}:{w.revision+1}:{kind}"
    t = replace(w,revision=w.revision+1)
    if kind == "transport":
        if action["species"] not in ("S","Sm") or action["direction"] not in ("in","out"):
            raise Rejected("declared finite transported species/direction required")
        q = finite(action["amount"],positive=True)*(1 if action["direction"] == "in" else -1)
        j = 4 if action["species"] == "S" else 5
        n[j] += q
        t = replace(t,**{("outside_s" if j==4 else "outside_sm"):(w.outside_s if j==4 else w.outside_sm)-q})
        cells[idx] = replace(c,n=tuple(n))
    elif kind in ("growth","maturation","remesh"):
        i = action["edge"]
        if type(i) is not int or not 0 <= i < len(edges):
            raise Rejected("edge index required")
        e = edges[i]
        if kind == "growth":
            q = finite(action["extent"])
            if q == 0:
                raise Rejected("nonzero growth extent required")
            n[5] -= q
            n[4] += (1-p.yield_y)*q
            edges[i] = _change_y(e,p.yield_y*q,label)
        elif kind == "maturation":
            q = finite(action["extent"])
            donor = e.young if q >= 0 else e.mature
            if q == 0 or abs(q) > donor:
                raise Rejected("maturation donor exhausted/zero extent")
            edges[i] = replace(e,tags=tuple(replace(tag,
                young=tag.young-q*((tag.young if q>=0 else tag.mature)/donor),
                mature=tag.mature+q*((tag.young if q>=0 else tag.mature)/donor)) for tag in e.tags))
        else:
            a,b = c.vertices[i],c.vertices[(i+1)%len(edges)]
            mid = ((a[0]+b[0])/2,(a[1]+b[1])/2)
            half = tuple(replace(tag,fraction=tag.fraction/2,young=tag.young/2,mature=tag.mature/2) for tag in e.tags)
            edges[i:i+1] = [Edge(e.id+".0",half),Edge(e.id+".1",half)]
            vertices = c.vertices[:i+1]+(mid,)+c.vertices[i+1:]
            cells[idx] = replace(c,vertices=vertices,edges=tuple(edges))
        if kind != "remesh":
            cells[idx] = replace(c,n=tuple(n),edges=tuple(edges))
    else:
        i,j = action["i"],action["j"]
        if type(i) is not int or type(j) is not int or not 0 <= i < j < len(edges) or j-i < 2 or len(edges)-(j-i) < 2:
            raise Rejected("valid two-loop cut required")
        distance = math.dist(c.vertices[i],c.vertices[j])
        mass = p.rho*distance
        q = 2*mass/p.yield_y
        n[5] -= q
        n[4] += (1-p.yield_y)*q
        children = []
        for number, indices in enumerate((tuple(range(i,j+1)),tuple(range(j,len(edges)))+tuple(range(i+1)))):
            vertices = tuple(c.vertices[k] for k in indices)
            inherited = tuple(edges[k] for k in indices[:-1])
            closure = Edge(label+f":closure:{number}",(Tag(label+f":closure:{number}",1.,mass,0.),))
            fraction = area(vertices)/area(c.vertices)
            children.append(Cell(c.id+f"/{w.revision+1}:{number}",vertices,inherited+(closure,),
                                 tuple(x*fraction for x in n),c.P*fraction,c.a0*fraction,c.R_ref*fraction))
        if abs(sum(area(d.vertices) for d in children)-area(c.vertices)) > 1e-10:
            raise Rejected("daughter area inconsistency")
        cells[idx:idx+1] = children
    t = replace(t,cells=tuple(cells))
    validate(t)
    before, after = invariants(w), invariants(t)
    if any(abs(before[k]-after[k]) > 1e-10 for k in before):
        raise Rejected("event source/material inheritance defect")
    return t


@dataclass(frozen=True)
class Proposal:
    owner: str
    before_hash: str
    action_json: str
    after_hash: str


@dataclass(frozen=True)
class Receipt:
    owner: str
    preimage: World
    before_hash: str
    after_hash: str
    action_json: str


def propose(w, action):
    try:
        action_json = json.dumps(action,sort_keys=True,separators=(",",":"),allow_nan=False)
    except (TypeError,ValueError) as error:
        raise Rejected("nonfinite/unserializable event") from error
    target = _event(w,json.loads(action_json))
    return Proposal(w.owner,digest(w),action_json,digest(target))


def commit(w, proposal):
    if type(proposal) is not Proposal or proposal.owner != w.owner or proposal.before_hash != digest(w):
        raise Rejected("stale/foreign proposal")
    target = _event(w,json.loads(proposal.action_json))
    if digest(target) != proposal.after_hash:
        raise Rejected("proposal target binding defect")
    before, after = energy(w),energy(target)
    def components(g):
        return {"bulk_ideal":g["outside"]+sum(c["g_source"] for c in g["cells"]),
                "solid_standard":sum(c["g_bound"] for c in g["cells"]),
                "native_elastic":sum(c["elastic"] for c in g["cells"]),
                "new_coupling":sum(c["coupling"] for c in g["cells"])}
    cb, ca = components(before), components(after)
    changes = {key:ca[key]-cb[key] for key in cb}
    record = {"action":json.loads(proposal.action_json),"before":asdict(w),"after":asdict(target),
              "energy_before":before,"energy_after":after,"pair":pair_rates(after["total"]-before["total"],w.params),
              "invariants_before":invariants(w),"invariants_after":invariants(target),
              "energy_changes":changes,"chemical_free_energy_release":-changes["bulk_ideal"]-changes["solid_standard"],
              "mechanical_storage_change":changes["native_elastic"]+changes["new_coupling"],
              "thermal_boundary":"fixed-theta isothermal reservoir, distinct from finite material bath; no heat/closed-first-law claim",
              "interpretation":"ensemble event-pair entropy; no trajectory heat or physical fusion claim"}
    receipt = Receipt(w.owner,w,digest(w),digest(target),proposal.action_json)
    return target,receipt,record


def reverse(w, receipt):
    if type(receipt) is not Receipt or receipt.owner != w.owner or receipt.after_hash != digest(w) or receipt.before_hash != digest(receipt.preimage):
        raise Rejected("stale/foreign/corrupted inverse receipt")
    if receipt.preimage.owner != w.owner or receipt.preimage.params != w.params:
        raise Rejected("inverse physical owner changed")
    check = _event(receipt.preimage,json.loads(receipt.action_json))
    if digest(check) != receipt.after_hash:
        raise Rejected("inverse proposal/preimage mismatch")
    return validate(replace(receipt.preimage,revision=w.revision+1))


def prepared_world(params=None, owner="reference-root"):
    p = Params() if params is None else params
    vertices = tuple((math.cos(i*math.pi/3),math.sin(i*math.pi/3)) for i in range(6))
    ls = tuple(math.dist(a,b) for a,b in zip(vertices,vertices[1:]+vertices[:1]))
    edges = tuple(Edge(f"edge:{i}",(Tag(f"birth:{i}",1.,0.1,L*p.rho),)) for i,L in enumerate(ls))
    c = Cell("cell:0",vertices,edges,(0.1,0.05,0.05,0.01,1.,10.),0.2,0.1,sum(ls))
    return validate(World(owner,p,area(vertices),(c,),10.,10.,10.))


def fixture_campaign(native=None):
    w = prepared_world()
    actions = [{"kind":"transport","cell":"cell:0","species":"Sm","direction":"in","amount":0.2},
               {"kind":"growth","cell":"cell:0","edge":0,"extent":0.2},
               {"kind":"maturation","cell":"cell:0","edge":0,"extent":0.04},
               {"kind":"fission","cell":"cell:0","i":0,"j":3},
               {"kind":"remesh","cell":"cell:0","edge":0}]
    events = []
    for action in actions:
        target,receipt,record = commit(w,propose(w,action))
        recovered = reverse(target,receipt)
        record["inverse_physical_recovery"] = digest(recovered,True)==digest(w,True)
        record["receipt"] = {"owner":receipt.owner,"before_hash":receipt.before_hash,"after_hash":receipt.after_hash,
                             "action_json":receipt.action_json,"preimage_is_record_before":True}
        record["recovered"] = asdict(recovered)
        record["inverse_pair"] = pair_rates(-record["pair"]["delta_g"],w.params)
        events.append(record)
    loaded = _event(w,actions[2])
    _,_,loaded_remesh = commit(loaded,propose(loaded,actions[4]))
    drive = []
    action = actions[2]
    for mu in (-50.,50.,None):
        if mu is None:
            base = prepared_world(replace(w.params,mu_m=0.))
            trial = _event(base,action)
            mu = -(energy(trial)["total"]-energy(base)["total"])/action["extent"]
        control = prepared_world(replace(w.params,mu_m=mu))
        _,_,record = commit(control,propose(control,action))
        drive.append(record)
    grid = []
    for h,y in ((0.,1.),(0.0005,0.001),(0.01,0.002),(1.,0.2)):
        for L in (0.00025,0.002,0.01,0.5,2.,8.):
            U,F = elastic(L,h,y,w.params)
            grid.append({"L":L,"mature":h,"young":y,"energy":U,"force":F})
    L,h,y = 0.004,0.0005,0.0005
    old = elastic(L,h,y,w.params)[0]
    new = 2*elastic(L/2,h/2,y/2,w.params)[0]
    rejections = []
    for action in ({**actions[0],"amount":1000.},{**actions[1],"extent":1000.},{**actions[2],"extent":10.}):
        saved = digest(w)
        try:
            propose(w,action)
        except Rejected as error:
            rejections.append({"reason":str(error),"unchanged":saved==digest(w)})
        else:
            raise AssertionError("insufficient donor accepted")
    target,receipt,_ = commit(w,propose(w,actions[1]))
    for other in (w,replace(target,owner="foreign-owner")):
        saved = digest(other)
        try:
            reverse(other,receipt)
        except Rejected as error:
            rejections.append({"reason":str(error),"unchanged":saved==digest(other)})
        else:
            raise AssertionError("stale/foreign inverse accepted")
    return {"scope":"designed finite-event four-operator chemical work ownership reference",
            "identified_biological_mechanism":False,"native_full_integration":False,"physical_fusion":False,
            "params":asdict(w.params),"events":events,"loaded_neutral_remesh":loaded_remesh,"drive_controls":drive,"force_grid":grid,
            "floor_remesh_counterexample":{"L":L,"mature":h,"young":y,"before":old,"after":new,"delta":new-old},
            "rejections":rejections,"native_audit":native}


if __name__ == "__main__":
    print(json.dumps(fixture_campaign(),sort_keys=True,allow_nan=False,separators=(",",":")))
