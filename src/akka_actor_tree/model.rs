use crate::ui::formatter::TreeWidgetNode;

#[derive(Clone)]
pub struct AkkaActorTreeSettings {
    pub tree_address: String,
    pub count_address: String,
    pub tree_timeout: u64,
    pub count_timeout: u64,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ActorNode {
    pub name: String,
    pub parent: Option<usize>,
    pub id: usize,
}

impl TreeWidgetNode for ActorNode {
    fn id(&self) -> usize {
        self.id
    }

    fn parent_id(&self) -> Option<usize> {
        self.parent
    }

    fn label(&self) -> String {
        self.name.to_owned()
    }
}
