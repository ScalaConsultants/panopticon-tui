use std::fmt;
use serde::Deserialize;

#[derive(Clone)]
pub struct AkkaClusterSettings {
    pub cluster_status_address: String,
}

#[derive(Deserialize, Debug)]
pub struct ClusterStatus {
    #[serde(rename(deserialize = "selfNode"))]
    pub self_node: String,
    pub members: Vec<ClusterMember>,
    pub unreachable: Vec<String>,
    pub leader: String,
    pub oldest: String,
}

#[derive(Deserialize, Debug)]
pub struct ClusterMember {
    pub node: String,
    #[serde(rename(deserialize = "nodeUid"))]
    pub node_uid: String,
    pub status: String,
    pub roles: Vec<String>,
}

impl fmt::Display for ClusterMember {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "==> node:   {}\n    uid:    {}\n    status: {}\n    roles:   {}", self.node, self.node_uid, self.status, self.roles.join(", "))
    }
}
