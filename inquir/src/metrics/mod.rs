use crate::ast::{
    Expr,
    System, LocExpr,
};

pub struct Metrics {
    e_depth: u32,
    e_count: u32,
}

impl Metrics {
    pub fn new(s: &System) -> Self {
        Self {
            e_depth: calc_e_depth(s),
            e_count: calc_e_count(s),
        }
    }

    pub fn e_count(&self) -> u32 {
        self.e_count
    }

    pub fn e_depth(&self) -> u32 {
        self.e_depth
    }
}

pub fn calc_e_depth(s: &System) -> u32 {
    match s {
        System::Located(LocExpr { p: _, exps }) => exps.iter().fold(0, |sum, e| sum + calc_e_depth_exp(e)),
        System::Composition(ss) => ss.iter().map(|t| calc_e_depth(t)).max().unwrap(),
    }
}

fn calc_e_depth_exp(e: &Expr) -> u32 {
    match e {
        Expr::GenEnt(_) => 1,
        Expr::Parallel(es) => es.iter().map(|e| calc_e_depth_exp(e)).max().unwrap(),
        _ => 0,
    }
}

pub fn calc_e_count(s: &System) -> u32 {
    match s {
        System::Located(LocExpr { p: _, exps }) =>  exps.iter().map(|e| calc_e_count_exp(e)).sum(),
        System::Composition(ss) => ss.iter().map(|t| calc_e_count(t)).sum(),
    }
}

fn calc_e_count_exp(e: &Expr) -> u32 {
    match e {
        Expr::GenEnt(_) => 1,
        Expr::Parallel(es) => es.iter().map(|e| calc_e_count_exp(e)).sum(),
        _ => 0,
    }
}
