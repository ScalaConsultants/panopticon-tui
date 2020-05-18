#[derive(Clone)]
pub struct AkkaSettings {
    pub tree_address: String,
    pub count_address: String,
    pub tree_timeout: u64,
    pub count_timeout: u64,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ActorTreeNode {
    pub name: String,
    pub parent: Option<usize>,
    pub id: usize,
}
