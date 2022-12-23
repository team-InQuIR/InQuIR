use inquir::{
    Process,
    System, LocProc,
};

use crate::simulation::evaluation_cost::EvaluationCost;

use serde::Serialize;


#[derive(Serialize, Debug)]
pub struct Metrics {
    c_depth: u32,
    c_count: u32,
    e_depth: u32,
    e_count: u32,
    time: u32,
}

impl Metrics {
    pub fn new(s: &System, eval_cost: EvaluationCost) -> Self {
        Self {
            c_depth: eval_cost.c_depth(),
            c_count: calc_c_count(s),
            e_depth: eval_cost.e_depth(),
            e_count: calc_e_count(s),
            time: eval_cost.time(),
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

    pub fn time(&self) -> u32 {
        self.time
    }
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
