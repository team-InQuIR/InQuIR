use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BExpr {
    True,
    False,
    Var(String),
    Not(Box<BExpr>),
    And(Box<BExpr>, Box<BExpr>),
    Or(Box<BExpr>, Box<BExpr>),
    Xor(Box<BExpr>, Box<BExpr>),
}

impl fmt::Display for BExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BExpr::True => write!(f, "1"),
            BExpr::False => write!(f, "0"),
            BExpr::Var(id)=> write!(f, "{}", id),
            BExpr::Not(b) => write!(f, "!{}", **b),
            BExpr::And(l, r) => write!(f, "{} & {}", **l, **r),
            BExpr::Or(l, r) => write!(f, "{} || {}", **l, **r),
            BExpr::Xor(l, r) => write!(f, "{} + {}", **l, **r),
        }
    }
}
