use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Fiber {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub status: FiberStatus,
    pub dump: String,
}

impl Display for Fiber {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{} {:?} {}", self.id, self.parent_id, self.status)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum FiberStatus {
    Done,
    Finishing,
    Running,
    Suspended,
}

impl Display for FiberStatus {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}
