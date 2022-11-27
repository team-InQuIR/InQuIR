use graph::graph::{DiGraph, NodeIndex};
use inquir::{
    ProcessorId,
    Expr,
    InitExpr, GenEntExpr, EntSwapExpr, RCXCExpr, RCXTExpr,
    QSendExpr, QRecvExpr, SendExpr, RecvExpr, ApplyExpr, MeasureExpr,
    System, LocExpr,
};
use std::collections::BTreeMap;

// TODO: Distinguish edges between read edges and write edges.
pub type DependencyGraph = DiGraph<(ProcessorId, Expr), ()>;

pub struct DependencyGraphBuilder {
    g: DependencyGraph,
    last_node_id: BTreeMap<String, NodeIndex>,
}

impl DependencyGraphBuilder {
    pub fn new() -> Self {
        Self {
            g: DependencyGraph::new(),
            last_node_id: BTreeMap::new(),
        }
    }

    pub fn build(mut self, s: System) -> DependencyGraph {
        self.add_system(s);
        self.g
    }

    fn add_system(&mut self, s: System) {
        match s {
            System::Located(LocExpr { p, exps }) => exps.into_iter().for_each(|e| self.add_exp(p, e)),
            System::Composition(ss) => ss.into_iter().for_each(|s| self.add_system(s)),
        }
    }

    fn add_exp(&mut self, p: ProcessorId, e: Expr) {
        match e {
            Expr::Skip => {},
            Expr::Init(InitExpr { dst }) => {
                let id = self.g.add_node((p, Expr::Init(InitExpr { dst: dst.clone() })));
                self.last_node_id.insert(dst, id);
            },
            Expr::Free(_) => {
                let _ = self.g.add_node((p, e));
            },
            Expr::GenEnt(GenEntExpr { label, partner }) => {
                let id = self.g.add_node((p, Expr::GenEnt(GenEntExpr { label: label.clone(), partner })));
                self.last_node_id.insert(label, id);
            },
            Expr::EntSwap(EntSwapExpr { ref arg1, ref arg2 }) => {
                let ch1 = arg1.clone();
                let ch2 = arg2.clone();
                let id = self.g.add_node((p, e));
                let id1 = self.last_node_id[&ch1];
                let id2 = self.last_node_id[&ch2];
                let _ = self.g.add_edge(id1, id, ());
                let _ = self.g.add_edge(id2, id, ());
                *self.last_node_id.get_mut(&ch1).unwrap() = id;
                *self.last_node_id.get_mut(&ch2).unwrap() = id;
            },
            // Compilers must decompose these two instructions!
            Expr::RCXC(RCXCExpr { arg, ent }) => {
                let from_arg = self.last_node_id[&arg];
                let from_ent = self.last_node_id[&ent];
                let id = self.g.add_node((p, Expr::RCXC(RCXCExpr { arg: arg.clone(), ent })));
                let _ = self.g.add_edge(from_arg, id, ());
                let _ = self.g.add_edge(from_ent, id, ());
                *self.last_node_id.get_mut(&arg).unwrap() = id;
                // discard `ent` here
            },
            Expr::RCXT(RCXTExpr { arg, ent }) => {
                let from_arg = self.last_node_id[&arg];
                let from_ent = self.last_node_id[&ent];
                let id = self.g.add_node((p, Expr::RCXT(RCXTExpr { arg: arg.clone(), ent })));
                let _ = self.g.add_edge(from_arg, id, ());
                let _ = self.g.add_edge(from_ent, id, ());
                *self.last_node_id.get_mut(&arg).unwrap() = id;
                // discard `arg` and `ent` here
            },
            Expr::QSend(QSendExpr { arg, ent }) => {
                let from_arg = self.last_node_id[&arg];
                let from_ent = self.last_node_id[&ent];
                let id = self.g.add_node((p, Expr::QSend(QSendExpr { arg: arg.clone(), ent })));
                let _ = self.g.add_edge(from_arg, id, ());
                let _ = self.g.add_edge(from_ent, id, ());
                *self.last_node_id.get_mut(&arg).unwrap() = id;
                // discard `ent` here
            },
            Expr::QRecv(QRecvExpr { dst, ent }) => {
                let from_ent = self.last_node_id[&ent];
                let id = self.g.add_node((p, Expr::QRecv(QRecvExpr { dst: dst.clone(), ent })));
                let _ = self.g.add_edge(from_ent, id, ());
                self.last_node_id.insert(dst, id);
                // discard `ent` here
            },
            Expr::Send(SendExpr { ch, data }) => todo!(),
            Expr::Recv(RecvExpr { ch, data }) => todo!(),
            Expr::Apply(ApplyExpr { gate, args }) => {
                let froms: Vec<_> = args.iter().map(|x| self.last_node_id[x]).collect();
                let id = self.g.add_node((p, Expr::Apply(ApplyExpr { gate, args: args.clone() })));
                froms.into_iter().for_each(|from| {
                    let _ = self.g.add_edge(from, id, ());
                });
                args.into_iter().for_each(|x| {
                    *self.last_node_id.get_mut(&x).unwrap() = id;
                });
            },
            Expr::Measure(MeasureExpr { dst, args }) => {
                let id = self.g.add_node((p, Expr::Measure(MeasureExpr { dst: dst.clone(), args: args.clone() })));
                for x in args {
                    let from = self.last_node_id[&x];
                    let _ = self.g.add_edge(from, id, ());
                    *self.last_node_id.get_mut(&x).unwrap() = id;
                }
                self.last_node_id.insert(dst, id);
            },
            Expr::Parallel(_) => todo!(), // Really?: es.into_iter().for_each(|e| self.add_exp(p, e)),

        }
    }
}
