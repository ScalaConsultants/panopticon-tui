
use bytes::BytesMut;
use redis_protocol::prelude::*;
use redis_protocol::types::Frame;

use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use std::collections::hash_map::HashMap;
use std::error::Error;

use crate::zio::model::{Fiber, FiberStatus};

#[tokio::main]
pub async fn get_dump() -> Result<HashMap<Fiber, String>, Box<dyn Error>> {
    println!("get_dump");
    let frame = Frame::Array(vec![Frame::BulkString("dump".into())]);
    let mut buf = BytesMut::new();

    let _ = match encode_bytes(&mut buf, &frame) {
        Ok(l) => l,
        Err(e) => panic!("Error encoding frame: {:?}", e)
    };

    let mut stream = TcpStream::connect("127.0.0.1:1111").await?;

    let _ = stream.write(&buf).await;

    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).await?;

    let buf: BytesMut = buffer.into();

    let (frame, consumed) = match decode_bytes(&buf) {
        Ok((f, c)) => (f, c),
        Err(e) => panic!("Error parsing bytes: {:?}", e)
    };

    let mut fibers: HashMap<Fiber, String> = HashMap::new();

    if let Some(Frame::Array(frames)) = frame {    
        let v: Vec<(Fiber, &str)> = frames.iter().map(|f| {
            let dump = f.as_str().unwrap();
            let id = parse_fiber(dump.to_string()).unwrap();
            (id, dump)
        }).collect();
        v.iter().for_each(|f| {
            fibers.insert(f.0.clone(), f.1.to_string());
            ()
        });
        //println!("Parsed frame {:?} and consumed {} bytes", v, consumed);
    } else {
        println!("Incomplete frame, parsed {} bytes", consumed);
    }

    Ok(fibers)
}

//
// Takes a fiber dump string and returns a fiber is.
//
// Expects a string where the first line is of the following format:
//   #4 (7h432m25965s25965835ms)
//
fn parse_fiber_id(dump: String) -> String {
    let n: usize = dump.find(" ").unwrap();
    dump[..n].to_string()
}

//
// Takes a fiber dump string and parses it into Fiber model.
//
// Expects a string where two first lines are of the following format:
//   #4 (7h432m25965s25965835ms)
//   Status: Running()
//
fn parse_fiber(dump: String) -> Option<Fiber> {
    println!("parse_fiber");
    let fib_str: Vec<&str> = dump.lines().take(2).collect();

    let id_and_life: Option<(String, String)> = fib_str[0].trim().find(" ").map(|n| {
        let id = fib_str[0][..n].to_string();
        let life = fib_str[0][n..].to_string();
        (id, life)    
    });

    let status_line = fib_str[1];
    
    let status: Option<FiberStatus> = if status_line.contains("Done") {
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

    match (id_and_life, status) {
        (Some((id, life)), Some(status)) => Some(Fiber {
            id: id,
            life: life,
            status: status,
        }),
        _ => None
    }

    // Some(Fiber {
    //     id: "1".to_string(),
    //     life: "123".to_string(),
    //     status: FiberStatus::Running,
    // })
}
