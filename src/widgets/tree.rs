use std::collections::hash_map::HashMap;
use crate::zio::model::Fiber;
use crate::akka::model::ActorTreeNode;

pub trait TreeWidgetNode {
    fn id(&self) -> usize;
    fn parent_id(&self) -> Option<usize>;
    fn label(&self) -> String;
}

impl TreeWidgetNode for Fiber {
    fn id(&self) -> usize {
        self.id
    }

    fn parent_id(&self) -> Option<usize> {
        self.parent_id
    }

    fn label(&self) -> String {
        format!("{:?}", self.status)
    }
}

impl TreeWidgetNode for ActorTreeNode {
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

///
/// Given a list of items returns a list of formatted labels ready to be printed as a tree.
///
/// eg. A list of items:
///       (id: 0, parent_id: None, label: Suspended)
///       (id: 1, parent_id: 0,    label: Running)
///       (id: 2, parent_id: 1,    label: Suspended)
///       (id: 3, parent_id: 1,    label: Running)
///       (id: 4, parent_id: 1,    label: Suspended)
///       (id: 5, parent_id: 4,    label: Running)
///       (id: 6, parent_id: None, label: Suspended)
///       (id: 7, parent_id: None, label: Running)
/// should give:
/// ├#0     Suspended
/// │└#1    Running
/// │ ├#2   Suspended
/// │ ├#3   Running
/// │ └#4   Suspended
/// │  └#5  Running
/// ├#7     Running
/// └#6     Suspended
///
pub fn tree_list_widget<T: Clone + TreeWidgetNode>(items: Vec<T>, print_ids: bool) -> Vec<(String, T)> {
    let tree = &make_tree(items);

    // get the initial printable tree
    let temp: Vec<(String, T)> = match tree.get(&None) {
        Some(v) => list_tree_nodes(v.to_vec(), 0, tree, "".to_string(), print_ids),
        None => vec![]
    };

    // find the max length of the line to calculate padding
    let max_len = &temp.iter()
        .max_by_key(|i| i.0.chars().count())
        .map_or(0, |i| i.0.chars().count());


    // add label using padding
    temp.iter().map(|i| {
        (format!("{:width$} {}", i.0, i.1.label(), width = max_len), i.1.to_owned())
    }).collect()
}

///
/// Formats a tree of items as an ASCII tree.
/// The output is a vector of formatted label and a corresponding item (for further processing)
///
/// Given the input of:
///    items: 0, 6, 7
///    and
///    a tree:
///        1 -> (4,2,3)
///        0 -> 1
///        _ -> (0,7,6)
///        4 -> 5
///
/// should give labels:
/// ├#0
/// │└#1
/// │ ├#2
/// │ ├#3
/// │ └#4
/// │  └#5
/// ├#7
/// └#6
///
fn list_tree_nodes<T: Clone + TreeWidgetNode>(
    items: Vec<T>,
    level: usize,
    tree: &HashMap<Option<usize>, Vec<T>>,
    indent: String,
    print_ids: bool) -> Vec<(String, T)> {
    let size = items.len();

    if size == 0 {
        vec![]
    } else {
        let i = items.last().unwrap();
        let printed_id = if print_ids { i.id().to_string() } else { "".to_string() };
        let parent: (String, T) = (format!("{:width$}└─#{}", indent.clone(), printed_id, width = level), i.to_owned());
        let mut last_node = tree.get(&Some(i.id())).map(|v|
            list_tree_nodes(v.to_vec(), level + 1, tree, format!("{}  ", indent.clone()), print_ids)
        ).unwrap_or(vec![]);
        last_node.insert(0, parent);

        if items.len() > 1 {
            let new_indent = format!("{}│ ", indent.clone());
            let n = items.len() - 1;
            let mut all: Vec<(String, T)> = items[..n].iter().fold(vec![], |mut acc, i| {
                let printed_id = if print_ids { i.id().to_string() } else { "".to_string() };
                let parent: (String, T) = (format!("{:width$}├─#{}", indent.clone(), printed_id, width = level), i.to_owned());
                let mut nodes: Vec<(String, T)> = tree.get(&Some(i.id())).map(|v|
                    list_tree_nodes(v.to_vec(), level + 1, tree, new_indent.clone(), print_ids)
                ).unwrap_or(vec![]);
                nodes.insert(0, parent);
                acc.append(&mut nodes);
                acc
            });
            all.append(&mut last_node);
            all
        } else {
            last_node
        }
    }
}

///
/// Converts a list of items into a Map of (parent -> list of children).
/// The parent can be None, which indicates that the children are root nodes.
///
/// eg. A list of items:
///       (id: 0, parent_id: None)
///       (id: 1, parent_id: 0)
///       (id: 2, parent_id: 1)
///       (id: 3, parent_id: 1)
///       (id: 4, parent_id: 1)
///       (id: 5, parent_id: 4)
///       (id: 6, parent_id: None)
///       (id: 7, parent_id: None)
///    should return a following map:
///       1 -> (4,2,3)
///       0 -> 1
///       _ -> (0,7,6)
///       4 -> 5
fn make_tree<T: Clone + TreeWidgetNode>(items: Vec<T>) -> HashMap<Option<usize>, Vec<T>> {
    items.iter().fold(HashMap::new(), |mut acc, f| {
        let v = acc.entry(f.parent_id()).or_insert(vec![]);
        v.push(f.to_owned());
        acc
    })
}
