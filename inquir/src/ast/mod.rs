pub mod expr;
pub mod primitive_gate;
pub mod process;
pub mod system;

pub use expr::*;
pub use primitive_gate::*;
pub use process::*;
pub use system::*;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(String);
impl SessionId {
    pub fn new(s: String) -> Self {
        Self(s)
    }
    pub fn to_string(self) -> String {
        self.0.clone()
    }
}
impl fmt::Display for SessionId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParticipantId(u32);
impl ParticipantId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn to_u32(self) -> u32 {
        self.0
    }
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}
impl fmt::Display for ParticipantId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(String);
impl Label {
    pub fn new(l: String) -> Self {
        Self(l)
    }
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl fmt::Display for Label {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
