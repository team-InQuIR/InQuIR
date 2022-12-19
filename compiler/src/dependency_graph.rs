use graph::graph::{DiGraph, Node, Edge, NodeIndex, EdgeIndex};
use graph::algo::toposort;
use inquir::{
    SessionId,
    ParticipantId,
    Label,
    PrimitiveGate,
    Process,
    OpenProc, InitProc, FreeProc, GenEntProc, EntSwapProc,
    QSendProc, QRecvProc, SendProc, RecvProc, ApplyProc, MeasureProc,
    System, LocProc,
    Expr,
};
use std::collections::{HashMap, BTreeMap, VecDeque};

type NodeWeight = (ParticipantId, Process);
type DAGNode = Node<NodeWeight>;
type InnerGraph = DiGraph<NodeWeight, Dependency>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    label: String,
}

impl Dependency {
    pub fn new(label: String) -> Self {
        Self {
            label,
        }
    }

    pub fn label(&self) -> &String {
        &self.label
    }

    pub fn is_classical(&self) -> bool { // TODO
        &self.label[..2] == "_m"
    }
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    g: InnerGraph,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            g: InnerGraph::new(),
        }
    }

    pub fn remove_node(&mut self, idx: NodeIndex) {
        let e = &mut self.g.node_weight_mut(idx).1;
        // TODO: Currently, this method supports to remove single qubit gates only.
        match e {
            Process::Apply(ref mut e) => {
                if e.args.len() == 1 {
                    e.gate = PrimitiveGate::I;
                } else {
                    unimplemented!()
                }
            },
            _ => unimplemented!(),
        }
    }

    pub fn insert_app_after(&mut self, idx: NodeIndex, p: ParticipantId, app: ApplyProc) -> NodeIndex {
        // TODO: currently, this method supports to insert single qubit gates only.
        assert!(app.args.len() == 1);
        let var = app.args[0].clone();
        let new_node = self.g.add_node((p, Process::Apply(app)));
        let mut to = None;
        for &eidx in self.outgoing_edges(idx) {
            if *self.edge(eidx).weight().label() == var {
                to = Some(self.g.edge(eidx).target());
                self.g.update_edge(eidx, idx, new_node, Dependency::new(var.clone()));
                break;
            }
        }
        if let Some(to) = to {
            self.g.add_edge(new_node, to, Dependency::new(var));
        } else { // if the node[idx] is the endpoint operation
            self.g.add_edge(idx, new_node, Dependency::new(var));
        }
        new_node
    }

    pub fn replace_stmt(&mut self, u: NodeIndex, exp: Process) {
        self.node_weight_mut(u).1 = exp;
    }

    /// Note: The gate of `u` must be single-qubit gate.
    pub fn swap_adjacent(&mut self, u: NodeIndex, v: NodeIndex) {
        // Note: Keep node indices!
        // data flow (old): -- eidx2 --> [u] -- eidx1 --> [v] -- eidx3 -->
        assert!(self.g.incoming_edges(v).iter().any(|&eidx| { u == self.g.edge(eidx).source() }));
        let eidx1 = *self.g.outgoing_edges(u).iter().find(|&&eidx| self.g.edge(eidx).target() == v).unwrap();
        let label = self.g.edge(eidx1).weight().label().clone();
        let eidx2 = self.find_depends_by_label(u, &label, true).unwrap(); // eidx2 must exist
        // Note: Must get eidx3 before update_edge!
        // Note: eidx3 may not exist (if [v] is the endpoint)
        let eidx3_opt = self.find_depends_by_label(v, &label, false);
        assert!(eidx3_opt != None
                || self.g.outgoing_edges(v).len() == 0 // single
                || self.g.incoming_edges(v).len() == 2 && self.g.outgoing_edges(v).len() == 1); // CX
        let dep_data = Dependency::new(label.clone());
        self.g.update_edge(eidx1, v, u, dep_data.clone());
        self.g.update_edge(eidx2, self.g.edge(eidx2).source(), v, dep_data.clone());
        if let Some(eidx3) = eidx3_opt {
            self.g.update_edge(eidx3, u, self.g.edge(eidx3).target(), dep_data);
        }
    }

    pub fn replace_gate(&mut self, u: NodeIndex, gate: PrimitiveGate) {
        let (_, e) = self.g.node_weight_mut(u);
        e.as_app_mut().unwrap().gate = gate;
    }

    fn find_depends_by_label(&self, u: NodeIndex, label: &String, is_in: bool) -> Option<EdgeIndex> {
        let edges = if is_in {
            self.g.incoming_edges(u)
        } else {
            self.g.outgoing_edges(u)
        };
        let index = edges.iter().position(|&eidx| self.g.edge(eidx).weight().label() == label);
        if let Some(index) = index {
            Some(edges[index])
        } else {
            None
        }
    }

    pub fn propagate_classical_deps(&mut self, u: NodeIndex, v: NodeIndex) {
        let cdeps1: Vec<_> = self.g.incoming_edges(u).clone().into_iter()
            .filter(|&eidx| self.g.edge(eidx).weight().is_classical()).collect(); // TODO: Replace is_classical
        cdeps1.into_iter().for_each(|eidx| {
            let already = self.g.incoming_edges(v).iter()
                .any(|&eidx2| self.g.edge(eidx2).weight().label() == self.g.edge(eidx).weight().label());
            if !already {
                let from = self.g.edge(eidx).source();
                let dep_data = Dependency::new(self.g.edge(eidx).weight().label().clone());
                self.g.add_edge(from, v, dep_data);
            }
        });
    }

    pub fn replace_bexp_until_end(&mut self, idx: NodeIndex, var: &String, exp: Expr) {
        // TODO: update classical dependencies?
        let (p, proc) = self.node(idx).weight();
        let proc = match proc {
            Process::Send(SendProc { s, dst, data: (label, exp) }) => {
                let exp = inquir::expr::subst_bexp(exp.clone(), var, exp.clone());
                Some(Process::Send(SendProc { s: s.clone(), dst: *dst, data: (label.clone(), exp) }))
            },
            Process::Apply(ApplyProc { gate, args, ctrl }) => {
                let ctrl = ctrl.clone().map(|b| inquir::expr::subst_bexp(b, var, exp.clone()));
                Some(Process::Apply(ApplyProc { gate: gate.clone(), args: args.clone(), ctrl }))
            },
            _ => None,
        };
        if let Some(proc) = proc {
            *self.node_weight_mut(idx) = (*p, proc);
        } else {
            let nexts: Vec<_> = self.g.outgoing_edges(idx).iter().
                filter_map(|&eidx| {
                    let edge = self.g.edge(eidx);
                    if edge.weight().label() == var {
                        Some(edge.target())
                    } else {
                        None
                    }
                }).collect();
            nexts.into_iter().for_each(|to| self.replace_bexp_until_end(to, var, exp.clone()));
        }
    }

    pub fn node_count(&self) -> usize {
        self.g.node_count()
    }

    pub fn node(&self, idx: NodeIndex) -> &DAGNode {
        self.g.node(idx)
    }

    pub fn edge(&self, idx: EdgeIndex) -> &Edge<Dependency> {
        self.g.edge(idx)
    }

    pub fn add_node(&mut self, w: NodeWeight) -> NodeIndex {
        self.g.add_node(w)
    }

    pub fn add_edge(&mut self, u: NodeIndex, v: NodeIndex, w: Dependency) -> EdgeIndex {
        self.g.add_edge(u, v, w)
    }

    pub fn incoming_edges(&self, idx: NodeIndex) -> &Vec<EdgeIndex> {
        self.g.incoming_edges(idx)
    }

    pub fn outgoing_edges(&self, idx: NodeIndex) -> &Vec<EdgeIndex> {
        self.g.outgoing_edges(idx)
    }

    pub fn node_weight_mut(&mut self, idx: NodeIndex) -> &mut NodeWeight {
        self.g.node_weight_mut(idx)
    }

    pub fn update_edge(&mut self, idx: EdgeIndex, u: NodeIndex, v: NodeIndex, w: Dependency) {
        self.g.update_edge(idx, u, v, w);
    }

    pub fn graph_ref(&self) -> &InnerGraph {
        &self.g
    }

    pub fn as_system(&self) -> System {
        self.check_cycle();
        // TODO: How to determine the order?
        let tord = toposort(&self.g).unwrap();
        let mut res = Vec::new();
        for idx in tord {
            let (p, exp) = self.node(idx).weight();
            if exp.is_app() && exp.as_app().unwrap().gate == PrimitiveGate::I {
                continue;
            }
            let p = p.to_u32() as usize;
            while res.len() <= p {
                res.push(Vec::new());
            }
            res[p].push(exp.clone());
        }

        let ss: Vec<_> = res.into_iter().enumerate().filter_map(|(p, procs)| {
            if procs.is_empty() {
                None
            } else {
                let p = ParticipantId::new(p as u32);
                Some(System::Located(LocProc { p, procs }))
            }
        }).collect();
        System::Composition(ss)
    }

    pub fn as_graphviz(&self) -> String {
        let mut res = String::new();
        res += "digraph dependency_graph {\n";
        res += "  graph [
                    charset = \"UTF-8\"
                    label = \"dependency graph\"
                  ]\n";
        (0..self.g.node_count()).for_each(|idx| {
            let (_, e) = self.g.node(idx).weight();
            res += &format!("node{} [label = \"{}: {}\"];\n", idx, idx, e);
        });
        self.g.edges().iter().for_each(|e| {
            let s = e.source();
            let t = e.target();
            res += &format!("node{} -> node{} [label = \"{}\"];\n", s, t, e.weight().label());
        });
        res += "}";

        res
    }

    // debug functions
    #[allow(dead_code)]
    fn debug_incoming_edges(&self, u: NodeIndex) {
        self.incoming_edges(u).iter().for_each(|&eidx| {
            dbg!(self.g.edge(eidx).weight());
        });
    }
    #[allow(dead_code)]
    fn debug_outgoing_edges(&self, u: NodeIndex) {
        self.outgoing_edges(u).iter().for_each(|&eidx| {
            dbg!(self.g.edge(eidx).weight());
        });
    }
    #[allow(dead_code)]
    fn check_cycle(&self) {
        let sz = self.g.node_count();
        let mut mark = vec![0; sz];
        for idx in 0..sz {
            let mut vs = VecDeque::new();
            let mut es = VecDeque::new();
            if let Some((vs, es)) = self.check_cycle_impl(idx, &mut vs, &mut es, &mut mark) {
                println!("---- cycle detected! -----------------------");
                vs.iter().for_each(|&idx| {
                    println!("node {}: {}", idx, self.g.node(idx).weight().1);
                });
                es.iter().for_each(|&idx| {
                    println!("edge {}: {}", idx, self.g.edge(idx).weight().label());
                });
                println!("--------------------------------------------");
                assert!(false);
                return;
            }
        }
    }
    #[allow(dead_code)]
    fn check_cycle_impl(
        &self,
        cur: NodeIndex,
        vs: &mut VecDeque<NodeIndex>,
        es: &mut VecDeque<EdgeIndex>,
        mark: &mut Vec<i32>
    ) -> Option<(VecDeque<NodeIndex>, VecDeque<EdgeIndex>)> {
        if mark[cur] == 2 { return None; }
        if mark[cur] == 1 { // detect cycle
            while *vs.front().unwrap() != cur {
                vs.pop_front();
                es.pop_front();
            }
            return Some((vs.clone(), es.clone()))
        }
        mark[cur] = 1;
        for &eidx in self.g.outgoing_edges(cur) {
            let to = self.g.edge(eidx).target();
            if mark[cur] == 2 { continue; }
            vs.push_back(to);
            es.push_back(eidx);
            if let Some(res) = self.check_cycle_impl(to, vs, es, mark) {
                return Some(res)
            }
        }
        mark[cur] = 2;
        None
    }
}

pub struct DependencyGraphBuilder {
    g: DependencyGraph,
    last_node_id: BTreeMap<String, NodeIndex>,
    sendrecv_pair: HashMap<(SessionId, Label), NodeIndex>, // TODO: unique?
    gen_ent_pair: HashMap<Label, Vec<NodeIndex>>,
    session_node_id: BTreeMap<(SessionId, ParticipantId), NodeIndex>,
}

impl DependencyGraphBuilder {
    pub fn new() -> Self {
        Self {
            g: DependencyGraph::new(),
            last_node_id: BTreeMap::new(),
            sendrecv_pair: HashMap::new(),
            gen_ent_pair: HashMap::new(),
            session_node_id: BTreeMap::new(),
        }
    }

    pub fn build(mut self, s: System) -> DependencyGraph {
        self.add_system(s);

        // Add virtual dependency between entanglement generations
        for (_, vec) in self.gen_ent_pair {
            assert!(vec.len() == 2);
            let nexts = vec![self.g.graph_ref().outgoing_nodes(vec[0]), self.g.graph_ref().outgoing_nodes(vec[1])];
            for i in 0..2 {
                let from = vec[i];
                nexts[(i + 1) % 2].iter().for_each(|&to| {
                    let dep_data = Dependency::new("__vdep".to_string());
                    self.g.add_edge(from, to, dep_data);
                });
            }
        }

        self.g
    }

    fn add_system(&mut self, s: System) {
        match s {
            System::Located(LocProc { p, procs }) => procs.into_iter().for_each(|proc| self.add_process(p, proc)),
            System::Composition(ss) => ss.into_iter().for_each(|s| self.add_system(s)),
        }
    }

    fn add_process(&mut self, p: ParticipantId, proc: Process) {
        match proc {
            Process::Open(OpenProc { id: sid, ps }) => {
                let id = self.g.add_node((p, Process::Open(OpenProc { id: sid.clone(), ps })));
                self.session_node_id.insert((sid, p), id);
            },
            Process::Init(InitProc { dst }) => {
                let id = self.g.add_node((p, Process::Init(InitProc { dst: dst.clone() })));
                self.last_node_id.insert(dst, id);
            },
            Process::Free(FreeProc { arg }) => {
                let prev_id = self.last_node_id[&arg];
                let id = self.g.add_node((p, Process::Free(FreeProc { arg: arg.clone() })));
                self.g.add_edge(prev_id, id, Dependency::new(arg));
            },
            Process::GenEnt(GenEntProc { x, p: another, label }) => {
                let id = self.g.add_node((p, Process::GenEnt(GenEntProc { x: x.clone(), p: another, label: label.clone() })));
                if !self.gen_ent_pair.contains_key(&label) {
                    self.gen_ent_pair.insert(label.clone(), Vec::new());
                }
                self.gen_ent_pair.get_mut(&label).unwrap().push(id);
                self.last_node_id.insert(x, id);
            },
            Process::EntSwap(EntSwapProc { ref x1, ref x2, ref arg1, ref arg2 }) => {
                let ch1 = arg1.clone();
                let ch2 = arg2.clone();
                let arg1_tmp = arg1.clone();
                let arg2_tmp = arg2.clone();
                let id = self.g.add_node((p, Process::EntSwap(EntSwapProc {
                    x1: x1.clone(), x2: x2.clone(), arg1: arg1.clone(), arg2: arg2.clone()
                })));
                let id1 = self.last_node_id[&ch1];
                let id2 = self.last_node_id[&ch2];
                let _ = self.g.add_edge(id1, id, Dependency::new(arg1_tmp));
                let _ = self.g.add_edge(id2, id, Dependency::new(arg2_tmp));
                *self.last_node_id.get_mut(&ch1).unwrap() = id;
                *self.last_node_id.get_mut(&ch2).unwrap() = id;
                self.last_node_id.insert(x1.to_string(), id);
                self.last_node_id.insert(x2.to_string(), id);
            },
            // Compilers must decompose these two instructions!
            Process::RCXC(_) => {
                unimplemented!()
                //let from_arg = self.last_node_id[&arg];
                //let from_ent = self.last_node_id[&ent];
                //let id = self.g.add_node((p, Proc::RCXC(RCXCProc { arg: arg.clone(), ent, uid })));
                //let _ = self.g.add_edge(from_arg, id, ());
                //let _ = self.g.add_edge(from_ent, id, ());
                //*self.last_node_id.get_mut(&arg).unwrap() = id;
                //// discard `ent` here
            },
            Process::RCXT(_) => {
                unimplemented!()
                //let from_arg = self.last_node_id[&arg];
                //let from_ent = self.last_node_id[&ent];
                //let id = self.g.add_node((p, Proc::RCXT(RCXTProc { arg: arg.clone(), ent, uid })));
                //let _ = self.g.add_edge(from_arg, id, ());
                //let _ = self.g.add_edge(from_ent, id, ());
                //*self.last_node_id.get_mut(&arg).unwrap() = id;
                //// discard `ent` here
            },
            Process::QSend(QSendProc { s, p: another, label, arg, ent, uid }) => {
                let from_arg = self.last_node_id[&arg];
                let from_ent = self.last_node_id[&ent];
                let id = self.g.add_node((p, Process::QSend(QSendProc { s, p: another, label, arg: arg.clone(), ent: ent.clone(), uid })));
                *self.last_node_id.get_mut(&arg).unwrap() = id;
                let _ = self.g.add_edge(from_arg, id, Dependency::new(arg));
                let _ = self.g.add_edge(from_ent, id, Dependency::new(ent));
                // discard `ent` here
            },
            Process::QRecv(QRecvProc { s, label, dst, ent, uid }) => {
                let from_ent = self.last_node_id[&ent];
                let id = self.g.add_node((p, Process::QRecv(QRecvProc { s, label, dst: dst.clone(), ent: ent.clone(), uid })));
                let _ = self.g.add_edge(from_ent, id, Dependency::new(ent));
                self.last_node_id.insert(dst, id);
                // discard `ent` here
            },
            Process::Send(SendProc { s, dst, data: (label, exp) }) => {
                let id = self.g.add_node((p, Process::Send(SendProc { s: s.clone(), dst, data: (label.clone(), exp.clone()) })));
                inquir::expr::variables(&exp).into_iter().for_each(|var| {
                    let from_data_id = self.last_node_id[&var];
                    let _ = self.g.add_edge(from_data_id, id, Dependency::new(var));
                });
                if let Some(recv_id) = self.sendrecv_pair.get(&(s.clone(), label.clone())) {
                    let _ = self.g.add_edge(id, *recv_id, Dependency::new("__comm_dep".to_string()));
                } else {
                    self.sendrecv_pair.insert((s, label), id);
                }
            },
            Process::Recv(RecvProc { s, data: (label, var) }) => {
                let id = self.g.add_node((p, Process::Recv(RecvProc { s: s.clone(), data: (label.clone(), var.clone()) })));
                self.last_node_id.insert(var, id);
                if let Some(send_id) = self.sendrecv_pair.get(&(s.clone(), label.clone())) {
                    let _ = self.g.add_edge(*send_id, id, Dependency::new("__comm_dep".to_string()));
                } else {
                    self.sendrecv_pair.insert((s, label), id);
                }
            },
            Process::Apply(ApplyProc { gate, args, ctrl }) => {
                let froms: Vec<_> = args.iter().map(|x| self.last_node_id[x]).collect();
                let id = self.g.add_node((p, Process::Apply(ApplyProc { gate, args: args.clone(), ctrl: ctrl.clone() })));
                for i in 0..froms.len() {
                    let _ = self.g.add_edge(froms[i], id, Dependency::new(args[i].clone()));
                }
                if let Some(vars) = ctrl.map(|e| self.collect_exp_vars(&e)) {
                    vars.into_iter().for_each(|x| {
                        let var_node = self.last_node_id[&x];
                        let _ = self.g.add_edge(var_node, id, Dependency::new(x));
                    });
                }
                args.into_iter().for_each(|x| {
                    *self.last_node_id.get_mut(&x).unwrap() = id;
                });
            },
            Process::Measure(MeasureProc { dst, args }) => {
                let id = self.g.add_node((p, Process::Measure(MeasureProc { dst: dst.clone(), args: args.clone() })));
                for x in args {
                    let from = self.last_node_id[&x];
                    *self.last_node_id.get_mut(&x).unwrap() = id;
                    let _ = self.g.add_edge(from, id, Dependency::new(x));
                }
                self.last_node_id.insert(dst, id);
            },
            Process::Parallel(procs) => procs.into_iter().for_each(|proc| self.add_process(p, proc)),
        }
    }

    fn collect_exp_vars(&self, e: &Expr) -> Vec<String> {
        match e {
            Expr::BLit(_) => Vec::new(),
            Expr::Var(id) => vec![id.clone()],
            Expr::Not(e) => self.collect_exp_vars(e),
            Expr::BinOp(_, l, r) => {
                let mut lvars = self.collect_exp_vars(&*l);
                let mut rvars = self.collect_exp_vars(&*r);
                lvars.append(&mut rvars);
                lvars
            },
        }
    }
}
