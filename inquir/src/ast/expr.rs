use std::fmt;
use std::collections::HashSet;

pub enum Value {
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    BLit(bool),
    Var(String),
    Not(Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    And,
    Or,
    Xor,
}

pub fn variables(b: &Expr) -> HashSet<String> {
    match b {
        Expr::BLit(_) => HashSet::new(),
        Expr::Var(var) => HashSet::from([var.clone()]),
        Expr::Not(b) => variables(b),
        Expr::BinOp(_, l, r) => {
            let mut vars = variables(l);
            vars.extend(variables(r));
            vars
        },
    }
}

pub fn subst_bexp(b1: Expr, id1: &String, b2: Expr) -> Expr {
    match b1 {
        Expr::Var(id2) if *id1 == id2 => b2,
        Expr::Not(b1) => subst_bexp(*b1, id1, b2),
        Expr::BinOp(op, l, r) => {
            let l = Box::new(subst_bexp(*l, id1, b2.clone()));
            let r = Box::new(subst_bexp(*r, id1, b2));
            Expr::BinOp(op, l, r)
        },
        b1 => b1,
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::BLit(true) => write!(f, "1"),
            Expr::BLit(false) => write!(f, "0"),
            Expr::Var(id)=> write!(f, "{}", id),
            Expr::Not(b) => write!(f, "!{}", **b),
            Expr::BinOp(op, l, r) => write!(f, "{} {} {}", **l, *op, **r),
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinOp::And => write!(f, "&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Xor => write!(f, "+"),
        }
    }
}
