use std::fmt;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Fiber {
    pub id: String,
    pub life: String,
    pub status: FiberStatus,
}

impl fmt::Display for Fiber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.id, self.life, self.status)
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
