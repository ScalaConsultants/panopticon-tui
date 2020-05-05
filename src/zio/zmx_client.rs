use bytes::BytesMut;
use redis_protocol::types::Frame;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use crate::zio::dump_parser;
use crate::zio::model::Fiber;

#[tokio::main]
pub async fn get_dump(addr: &str) -> Result<Vec<Fiber>, Box<dyn Error>> {
    let frame = Frame::Array(vec![Frame::BulkString("dump".into())]);
    let mut buf = BytesMut::new();

    let _ = match redis_protocol::prelude::encode_bytes(&mut buf, &frame) {
        Ok(l) => l,
        Err(e) => panic!("Error encoding frame: {:?}", e)
    };

    let mut stream = TcpStream::connect(addr).await?;

    let _ = stream.write(&buf).await;

    let mut buffer = String::new();
    stream.read_to_string(&mut buffer).await?;

    let buf: BytesMut = buffer.into();

    let (frame, consumed) = match redis_protocol::prelude::decode_bytes(&buf) {
        Ok((f, c)) => (f, c),
        Err(e) => panic!("Error parsing bytes: {:?}", e)
    };

    let mut fibers: Vec<Fiber> = vec![];

    if let Some(Frame::Array(frames)) = frame {
        let v: Vec<Fiber> = frames.iter().map(|f| {
            let dump = f.as_str().unwrap();
            dump_parser::parse_fiber_dump(dump.to_string()).unwrap()
        }).collect();
        v.iter().for_each(|f| {
            fibers.push(f.to_owned());
            ()
        });
    } else {
        println!("Incomplete frame, parsed {} bytes", consumed);
    }

    Ok(fibers)
}
