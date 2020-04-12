use std::fmt;

// Represents a mode that the node is in. Theoretically there are only three modes: leader, follower and standalone. 
// But since we only get a string from the server we can't really be sure if there's no error, 
// or some new mode has been introduced - that's why Unknown exists.
//
// On the other hand a Leader is a special node that returns some specific information. 
// That's why we need to be able to distinguish between them in the first place.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Follower,
    Leader,
    Standalone,
    Unknown,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub enum Word {
    Srvr,
    Conf,
    Wchc,
}

// pub struct KafkaCluster {
//     pub ids: Vec<String>,
//     pub topics: Vec<String>,
// }

#[derive(Debug)]
pub struct ZNode {
    pub id: String,
    pub mode: Mode,
}

pub struct ZAddr {
    pub host: String,
    pub port: String,
}
