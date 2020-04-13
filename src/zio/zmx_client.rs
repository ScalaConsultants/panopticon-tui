
use bytes::BytesMut;
use redis_protocol::prelude::*;
use redis_protocol::types::Frame;

use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use std::error::Error;

use crate::zio::model::{Fiber, FiberStatus};

#[tokio::main]
pub async fn get_dump() -> Result<Vec<Fiber>, Box<dyn Error>> {
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

    let mut fibers: Vec<Fiber> = vec![];

    if let Some(Frame::Array(frames)) = frame {    
        let v: Vec<Fiber> = frames.iter().map(|f| {
            let dump = f.as_str().unwrap();
            parse_fiber(dump.to_string()).unwrap()
        }).collect();
        v.iter().for_each(|f| {
            fibers.push(f.to_owned());
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
// fn parse_fiber_id(dump: String) -> String {
//     let n: usize = dump.find(" ").unwrap();
//     dump[..n].to_string()
// }

//
// Takes a fiber dump string and parses it into Fiber model.
//
// Expects a string where two first lines are of the following format:
//   #4 (7h432m25965s25965835ms)
//   Status: Running()
//
fn parse_fiber(dump: String) -> Option<Fiber> {
    let fib_str: Vec<&str> = dump.lines().take(2).collect();

    let id: Option<usize> = 
        fib_str[0].trim().find(" ").and_then(|n| {
            fib_str[0][1..n].parse::<usize>().ok()
        });

    let parent_id = dump.find("spawned").and_then(|n| {
        dump.get(n..).and_then(|sub| {
            let a = sub.find(",");
            let b = sub.find(")");
            match (a, b) {
                (Some(a), Some(b)) => sub.get(a+1..b).and_then(|i| i.parse::<usize>().ok()),
                _                  => None,
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
        (Some(id), Some(status)) => Some(Fiber {
            id: id,
            parent_id: parent_id,
            status: status,
            dump: dump,
        }),
        _ => None
    }
}