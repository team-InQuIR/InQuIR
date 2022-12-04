use std::fmt;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BExpr {
    True,
    False,
    Var(String),
    Not(Box<BExpr>),
    BinOp(BinOp, Box<BExpr>, Box<BExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    And,
    Or,
    Xor,
}

pub fn variables(b: &BExpr) -> HashSet<String> {
    match b {
        BExpr::True => HashSet::new(),
        BExpr::False => HashSet::new(),
        BExpr::Var(var) => HashSet::from([var.clone()]),
        BExpr::Not(b) => variables(b),
        BExpr::BinOp(_, l, r) => {
            let mut vars = variables(l);
            vars.extend(variables(r));
            vars
        },
    }
}

pub fn subst_bexp(b1: BExpr, id1: &String, b2: BExpr) -> BExpr {
    match b1 {
        BExpr::Var(id2) if *id1 == id2 => b2,
        BExpr::Not(b1) => subst_bexp(*b1, id1, b2),
        BExpr::BinOp(op, l, r) => {
            let l = Box::new(subst_bexp(*l, id1, b2.clone()));
            let r = Box::new(subst_bexp(*r, id1, b2));
            BExpr::BinOp(op, l, r)
        },
        b1 => b1,
    }
}

impl fmt::Display for BExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BExpr::True => write!(f, "1"),
            BExpr::False => write!(f, "0"),
            BExpr::Var(id)=> write!(f, "{}", id),
            BExpr::Not(b) => write!(f, "!{}", **b),
            BExpr::BinOp(op, l, r) => write!(f, "{} {} {}", **l, *op, **r),
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
