use bytes::BytesMut;
use redis_protocol::types::Frame;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use crate::zio::dump_parser;
use crate::zio::model::Fiber;

pub trait ZMXClient {
    fn address(&self) -> String;
    fn dump_fibers(&self) -> Result<Vec<Fiber>, String>;
}

pub struct NetworkZMXClient {
    address: String
}

impl NetworkZMXClient {
    pub fn new(address: String) -> NetworkZMXClient { NetworkZMXClient { address } }

    #[tokio::main]
    async fn get_dump(&self) -> Result<Vec<Fiber>, Box<dyn Error>> {
        let frame = Frame::Array(vec![Frame::BulkString("dump".into())]);
        let mut buf = BytesMut::new();

        let _ = match redis_protocol::prelude::encode_bytes(&mut buf, &frame) {
            Ok(l) => l,
            Err(e) => panic!("Error encoding frame: {:?}", e)
        };

        let mut stream = TcpStream::connect(&self.address).await?;

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
}

impl ZMXClient for NetworkZMXClient {
    fn address(&self) -> String {
        self.address.clone()
    }

    fn dump_fibers(&self) -> Result<Vec<Fiber>, String> {
        self.get_dump().map_err(|e| e.to_string())
    }
}

pub struct StubZMXClient {
    pub dump: Result<Vec<Fiber>, String>
}

impl StubZMXClient {
    pub fn new(dump: Result<Vec<Fiber>, String>) -> StubZMXClient { StubZMXClient { dump } }
}

impl ZMXClient for StubZMXClient {
    fn address(&self) -> String {
        "<stub>".to_owned()
    }

    fn dump_fibers(&self) -> Result<Vec<Fiber>, String> {
        self.dump.clone()
    }
}
