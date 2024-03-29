use std::fmt;
use crate::ast::ParticipantId;
use crate::ast::Process;

#[derive(Debug, Clone, PartialEq)]
pub enum System {
    /// a located expression `[e]p`.
    Located(LocProc),

    /// a composition of systems: `P1 | P2`.
    Composition(Vec<System>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocProc {
    pub p: ParticipantId,
    pub procs: Vec<Process>,
}

pub fn projection(s: &System, p: ParticipantId) -> Option<Vec<Process>> {
    match s {
        System::Located(proc) => {
            if proc.p == p {
                return Some(proc.procs.clone());
            } else {
                return None;
            }
        },
        System::Composition(ss) => {
            for s in ss {
                if let Some(procs) = projection(s, p) {
                    return Some(procs);
                }
            }
            None
        },
    }
}

impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            System::Located(LocProc { p, procs }) => {
                let mut s = String::new();
                s += format!("{} {{\n", p).as_str();
                procs.iter().for_each(|e| {
                    s += format!("  {};\n", e).as_str();
                });
                s += "\n}";
                write!(f, "{}", s)
            },
            System::Composition(ss) => {
                let s: Vec<_> = ss.iter().map(|s| format!("{}", s)).collect();
                let s = s.join("\n");
                write!(f, "{}", s)
            },
        }
    }
}

