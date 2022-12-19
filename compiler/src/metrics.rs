use inquir::{
    Process, FreeProc, GenEntProc, EntSwapProc,
    System, LocProc,
};
use graph::algo::toposort;

use crate::{
    arch::Configuration,
    dependency_graph::{DependencyGraph, DependencyGraphBuilder},
};

use std::collections::{VecDeque, BinaryHeap, HashMap};
use serde::Serialize;


#[derive(Serialize, Debug)]
pub struct Metrics {
    e_depth: u32, // estimated
    e_count: u32,
    c_depth: u32,
    c_count: u32,
}

impl Metrics {
    pub fn new(s: &System, config: &Configuration) -> Self {
        let builder = DependencyGraphBuilder::new();
        let g = builder.build(s.clone());

        Self {
            e_depth: calc_e_depth(&g, config),
            e_count: calc_e_count(s),
            c_depth: calc_c_depth(&g),
            c_count: calc_c_count(s),
        }
    }

    pub fn e_count(&self) -> u32 {
        self.e_count
    }

    pub fn e_depth(&self) -> u32 {
        self.e_depth
    }

    pub fn c_depth(&self) -> u32 {
        self.c_depth
    }

    pub fn c_count(&self) -> u32 {
        self.c_count
    }
}

pub fn calc_e_depth(g: &DependencyGraph, config: &Configuration) -> u32 {
    let sz = g.node_count();
    let p_sz = config.node_size();
    let mut dp = vec![0; sz];
    let mut in_deg: Vec<_> = (0..sz).map(|i| g.node(i).incoming().len()).collect();
    let mut que: VecDeque<_> = (0..sz).filter(|&i| g.node(i).incoming().is_empty()).collect();
    let mut ent_pool = vec![vec![BinaryHeap::new(); p_sz]; p_sz]; // cost
    config.connections().edges().iter().for_each(|e| {
        let s = e.source();
        let t = e.target();
        ent_pool[s][t].push(0);
        ent_pool[t][s].push(0);
    });
    let mut entanglements = vec![HashMap::new(); p_sz];
    while let Some(idx) = que.pop_front() {
        let (p, e) = g.node(idx).weight();
        let s = p.to_usize();
        let is_issued = match e {
            Process::GenEnt(GenEntProc { x, p: another, label: _ }) => {
                let t = another.to_usize();
                if let Some(cost) = ent_pool[s][t].pop() {
                    dp[idx] = cost + 1;
                    entanglements[s].insert(x, t);
                    true
                } else {
                    false
                }
            },
            Process::Free(FreeProc { arg }) => {
                let t = entanglements[s][&arg];
                ent_pool[s][t].push(dp[idx]); // return
                entanglements[s].remove(&arg);
                true
            },
            Process::EntSwap(EntSwapProc{ x1: _, x2: _, arg1, arg2 }) => {
                let t1 = entanglements[s][&arg1];
                let t2 = entanglements[s][&arg2];
                ent_pool[s][t1].push(dp[idx]);
                ent_pool[s][t2].push(dp[idx]);
                entanglements[s].remove(&arg1);
                entanglements[s].remove(&arg2);
                true
            },
            // These instructions have been decomposed.
            Process::QSend(_) => unimplemented!(),
            Process::QRecv(_) => unimplemented!(),
            Process::RCXC(_) => unimplemented!(),
            Process::RCXT(_) => unimplemented!(),
            Process::Parallel(_) => unimplemented!(),
            // Other instructions do not increase the E-depth.
            _ => true
        };
        if is_issued {
            g.node(idx).outgoing().iter().for_each(|&eidx| {
                let t = g.edge(eidx).target();
                in_deg[t] -= 1;
                if in_deg[t] == 0 {
                    que.push_back(t);
                }
                dp[t] = u32::max(dp[t], dp[idx]);
            });
        } else {
            que.push_back(idx);
        }
    }

    dp.into_iter().max().unwrap()
}

pub fn calc_e_count(s: &System) -> u32 {
    match s {
        System::Located(LocProc { p: _, procs }) =>  procs.iter().map(|proc| calc_e_count_proc(proc)).sum(),
        System::Composition(ss) => ss.iter().map(|t| calc_e_count(t)).sum(),
    }
}

fn calc_e_count_proc(proc: &Process) -> u32 {
    match proc {
        Process::GenEnt(_) => 1,
        Process::Parallel(procs) => procs.iter().map(|proc| calc_e_count_proc(proc)).sum(),
        _ => 0,
    }
}

fn calc_c_depth(g: &DependencyGraph) -> u32 {
    let sz = g.node_count();
    let mut dp = vec![0; sz];
    let tord = toposort(g.graph_ref()).unwrap();
    for idx in tord {
        let (_, exp) = g.node(idx).weight();
        match exp {
            Process::GenEnt(_) | Process::EntSwap(_)
                => g.outgoing_edges(idx).iter().for_each(|&eidx| {
                    let to = g.edge(eidx).target();
                    dp[to] = u32::max(dp[to], dp[idx]);
                }),
            Process::QSend(_) | Process::Send(_)
                => g.node(idx).outgoing().iter().for_each(|&eidx| {
                    let to = g.edge(eidx).target();
                    dp[to] = u32::max(dp[to], dp[idx] + 1)
                }),
            Process::QRecv(_) | Process::Recv(_)
                => g.node(idx).outgoing().iter().for_each(|&eidx| {
                    let to = g.edge(eidx).target();
                    dp[to] = u32::max(dp[to], dp[idx])
                }),
            Process::RCXC(_) | Process::RCXT(_)
                => g.node(idx).outgoing().iter().for_each(|&eidx| {
                    let to = g.edge(eidx).target();
                    dp[to] = u32::max(dp[to], dp[idx] + 1)
                }),
            Process::Parallel(_) => unimplemented!(),
            _ => g.node(idx).outgoing().iter().for_each(|&eidx| {
                let to = g.edge(eidx).target();
                dp[to] = u32::max(dp[to], dp[idx])
            }),
        }
    }
    dp.into_iter().max().unwrap()
}

fn calc_c_count(s: &System) -> u32 {
    match s {
        System::Located(LocProc { p: _, procs }) => procs.iter().map(|proc| calc_c_count_proc(proc)).sum(),
        System::Composition(ss) => ss.iter().map(|s| calc_c_count(s)).sum(),
    }
}

fn calc_c_count_proc(e: &Process) -> u32 {
    match e {
        Process::QSend(_) | Process::QRecv(_) => 1,
        Process::Send(_) | Process::Recv(_) => 1,
        Process::RCXC(_) | Process::RCXT(_) => 1,
        Process::Parallel(es) => es.iter().map(|e| calc_c_count_proc(e)).sum(),
        _ => 0,
    }
}
