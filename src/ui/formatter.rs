use std::collections::hash_map::HashMap;

use crate::zio::model::Fiber;
use crate::ui::model::UIFiber;

///
/// Given a list of fibers returns a list of formatted labels ready to be printed as a tree 
/// and a corrsponding fiber dumps.
///
/// eg. A list of fibers:
///       (id: 0, parent_id: None, status: Suspended)
///       (id: 1, parent_id: 0,    status: Running)
///       (id: 2, parent_id: 1,    status: Suspended)
///       (id: 3, parent_id: 1,    status: Running)
///       (id: 4, parent_id: 1,    status: Suspended)
///       (id: 5, parent_id: 4,    status: Running)
///       (id: 6, parent_id: None, status: Suspended)
///       (id: 7, parent_id: None, status: Running)
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
pub fn printable_tree(fibers: Vec<Fiber>) -> Vec<UIFiber> {
    let tree = &make_tree(fibers);

    // get the initial printable tree
    let temp: Vec<(String, Fiber)> = match tree.get(&None) {
        Some(v) => print_nodes(v.to_vec(), 0, tree, "".to_string()),
        None    => vec![]
    };

    // find the max length of the line to calculate padding
    let max_len = &temp.iter().fold(0, |acc, t| {
        if t.0.len() > acc {
            t.0.len()
        } else {
            acc
        }
    });

    // add fiber status to the label using padding
    temp.iter().map(|s| {
        UIFiber {
            label: format!("{:width$} {:?}", s.0, s.1.status, width = max_len),
            dump: s.1.dump.to_owned(),
        }
    }).collect()
}

//
// Formats a tree of Fibers as an ASCII tree. 
// The output is a vector of formatted label and a corrsponding Fiber (for further processing)
//
// Given the input of:
//    fibers: 0, 6, 7
//    and
//    a tree:
//        1 -> (4,2,3)
//        0 -> 1
//        _ -> (0,7,6)
//        4 -> 5
//
// should give labels:
// ├#0
// │└#1
// │ ├#2
// │ ├#3
// │ └#4
// │  └#5
// ├#7
// └#6
//
fn print_nodes(nodes: Vec<Fiber>, level: usize, tree: &HashMap<Option<usize>, Vec<Fiber>>, indent: String) -> Vec<(String, Fiber)> {
    let size = nodes.len();

    if size == 0 as usize {
        vec![]
    } else {
        let fib = nodes.last().unwrap();
        let parent: (String, Fiber) = (format!("{:width$}└─#{}", indent.clone(), fib.id, width = level), fib.to_owned());
        let mut last_node = tree.get(&Some(fib.id)).map(|v| 
            print_nodes(v.to_vec(), level + 1, tree, format!("{}  ", indent.clone()))
        ).unwrap_or_else(|| vec![]);
        last_node.insert(0, parent);

        if nodes.len() > 1 {
            let new_indent = format!("{}│ ", indent.clone());
            let n = nodes.len() - 1;
            let mut all: Vec<(String, Fiber)> = nodes[..n].iter().fold(vec![], |mut acc, fib| {
                let parent: (String, Fiber) = (format!("{:width$}├─#{}", indent.clone(), fib.id, width = level), fib.to_owned());
                let mut nodes: Vec<(String, Fiber)> = tree.get(&Some(fib.id)).map(|v| 
                    print_nodes(v.to_vec(), level + 1, tree, new_indent.clone())
                ).unwrap_or_else(|| vec![]);
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

//
// Converts a list of fibers into a Map of (parent -> list of children).
// The parent can be None, which indicatates that the children are root nodes.
//
// eg. A list of fibers:
//       (id: 0, parent_id: None)
//       (id: 1, parent_id: 0)
//       (id: 2, parent_id: 1)
//       (id: 3, parent_id: 1)
//       (id: 4, parent_id: 1)
//       (id: 5, parent_id: 4)
//       (id: 6, parent_id: None)
//       (id: 7, parent_id: None)
//    should return a following map:
//       1 -> (4,2,3)
//       0 -> 1
//       _ -> (0,7,6)
//       4 -> 5
fn make_tree(fibers: Vec<Fiber>) -> HashMap<Option<usize>, Vec<Fiber>> {
    // let test_tree: HashMap<Option<usize>, Vec<usize>> = [
    //     (Some(0), vec![1, 2, 3]),
    //     (Some(2), vec![4, 5]),
    //     (Some(5), vec![6, 11]),
    //     (Some(4), vec![7, 8]),
    //     (Some(6), vec![10]),
    //     (None, vec![0, 9])
    //     ].iter().cloned().collect();
    fibers.iter().fold(HashMap::new(), |mut acc, f| {
        let p_id = f.parent_id;
        let v = acc.entry(p_id).or_insert(vec![]);
        v.push(f.to_owned());
        acc
    })
}
