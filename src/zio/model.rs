use std::fmt;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Fiber {
    pub id: usize,
    pub parent_id: Option<usize>,
    pub status: FiberStatus,
    pub dump: String,
}

impl fmt::Display for Fiber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl fmt::Display for FiberStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
