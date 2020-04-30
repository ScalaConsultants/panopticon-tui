use crate::zio::dump_parser::parse_fiber_dump;
use crate::zio::model::{Fiber, FiberStatus};

#[test]
fn dump_parser_done() {
    let dump = "#4 (7h432m25965s25965835ms)
    Status: Done()";

    let result = Fiber {
        id: 4,
        parent_id: None,
        status: FiberStatus::Done,
        dump: dump.to_owned(),
    };
    assert_eq!(parse_fiber_dump(dump.to_owned()), Some(result));
}

#[test]
fn dump_parser_suspended_with_parent() {
    let dump = "#2 (1m98s98260ms) waiting on #2
    Status: Suspended(interruptible, 18 asyncs, zio.Promise.await(Promise.scala:50))
    <something>
    Fiber:Id(1588237280480,2) was spawned by:
    Fiber:Id(1588237280394,1) was supposed to continue to: ";

    let result = Fiber {
        id: 2,
        parent_id: Some(1),
        status: FiberStatus::Suspended,
        dump: dump.to_owned(),
    };
    assert_eq!(parse_fiber_dump(dump.to_owned()), Some(result));
}

#[test]
fn dump_parser_running() {
    let dump = "#3 (1m96s96402ms)
    Status: Running()";

    let result = Fiber {
        id: 3,
        parent_id: None,
        status: FiberStatus::Running,
        dump: dump.to_owned(),
    };
    assert_eq!(parse_fiber_dump(dump.to_owned()), Some(result));
}

#[test]
fn dump_parser_finishing() {
    let dump = "#3 (1m96s96402ms)
    Status: Finishing()";

    let result = Fiber {
        id: 3,
        parent_id: None,
        status: FiberStatus::Finishing,
        dump: dump.to_owned(),
    };
    assert_eq!(parse_fiber_dump(dump.to_owned()), Some(result));
}

#[test]
fn dump_parser_unknown_status() {
    let dump = "#3 (1m96s96402ms)
    Status: Trolling()";

    assert_eq!(parse_fiber_dump(dump.to_owned()), None);
}

#[test]
fn dump_parser_not_enough_lines() {
    assert_eq!(parse_fiber_dump("#3 (1m96s96402ms)".to_owned()), None);
    assert_eq!(parse_fiber_dump("".to_owned()), None);
}
