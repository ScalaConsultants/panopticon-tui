use crate::zio::model::{Fiber, FiberStatus};

///
/// Takes a fiber dump string and parses it into Fiber model.
///
/// Expects a string where two first lines are of the following format:
///
///   ```
///   #4 (7h432m25965s25965835ms)
///   Status: Running()
///   ```
pub fn parse_fiber_dump(dump: String) -> Option<Fiber> {
    let fib_str: Vec<&str> = dump.trim().lines().take(2).collect();

    if fib_str.len() < 2 {
        return None;
    }

    let id: Option<usize> =
        fib_str[0].trim().find(" ").and_then(|n| {
            fib_str[0][1..n].parse::<usize>().ok()
        });

    let parent_id = dump.find("spawned").and_then(|n| {
        dump.get(n..).and_then(|sub| {
            let a = sub.find(",");
            let b = sub.find(")");
            match (a, b) {
                (Some(a), Some(b)) => sub.get(a + 1..b).and_then(|i| i.parse::<usize>().ok()),
                _ => None,
            }
        })
    });

    let status_line = fib_str[1];

    let status: Option<FiberStatus> =
        if status_line.contains("Done") {
            Some(FiberStatus::Done)
        } else if status_line.contains("Finishing") {
            Some(FiberStatus::Finishing)
        } else if status_line.contains("Running") {
            Some(FiberStatus::Running)
        } else if status_line.contains("Suspended") {
            Some(FiberStatus::Suspended)
        } else {
            None
        };

    match (id, status) {
        (Some(id), Some(status)) => Some(Fiber { id, parent_id, status, dump }),
        _ => None
    }
}
