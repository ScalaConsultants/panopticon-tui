use std::collections::HashMap;
use std::env;
use std::fmt;
use std::process::Command;
use std::str;
use std::sync::mpsc;
use std::thread;
use crossterm::style::{style, Color, Attribute};

// Represents a mode that the node is in. Theoretically there are only to modes: leader and follower. 
// But since we only get a string from the server we can't really be sure if there's no error, 
// or some new mode has been introduced - that's why Unknown exists.
//
// On the other hand a Leader is a special node that returns some specific information. 
// That's why we need to able to distinguish between them in the first place.
#[derive(Clone, Copy, Debug)]
enum Mode {
    Follower,
    Leader,
    Standalone,
    Unknown,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct KafkaCluster {
    ids: Vec<String>,
    topics: Vec<String>,
}

struct ZNode {
    id: String,
    mode: Mode,
}

struct ZookeeperClient {
    host: String,
    port: String,
}

impl ZookeeperClient {

    fn new(host: String, port: String) -> ZookeeperClient {
        ZookeeperClient {
            host: host,
            port: port,
        }
    }

    fn call_nc(&self, command: &String, grep: &String) -> Vec<String> {
        let com = format!("echo -n '{}' | nc -w 3 {} {} | grep {}", command, self.host, self.port, grep);
    
        let output = Command::new("sh")
            .arg("-c")
            .arg(com)
            .output()
            .expect("no connection");
    
        let output_status = output.status;
    
        if output_status.success() {
            let mut output_std: Vec<u8> = output.stdout.clone();
            let pref_len = grep.len();

            let output_str = str::from_utf8(&output_std).unwrap();
            let output_str_lines = output_str.lines().map(|x| ZookeeperClient::remove_n(x.trim(), pref_len+1));
            
            let a = output_str_lines.filter(|x| x.is_some()).map(|x| x.unwrap().trim().to_string()).collect();
            println!("{:?}", a);
            return a;
        } else {
            return vec![];
        }
    }

    fn remove_first(s: &str) -> Option<&str> {
        s.chars().next().map(|c| &s[c.len_utf8()..])
    }

    fn remove_n(s: &str, n: usize) -> Option<String> {
        match s.char_indices().nth(n) {
            Some((n, _)) => {
                let a = s.to_string().drain(n..).collect::<String>();
                Some(a)
            },
            None         => None
        }
    }

    fn get_status(&self) -> Option<ZNode> {
        println!("get_status");
        let mut modes = self.call_nc(&"srvr".to_string(), &"Mode".to_string());
        let mut server_ids = self.call_nc(&"conf".to_string(), &"serverId".to_string());

        let mode = modes.pop();
        let server_id = server_ids.pop();

        let znode: Option<ZNode> = match (mode, server_id) {
            (Some(m), Some(id)) => {
                println!("{}", m);
                let mode = match m.as_str() {
                    "follower"   => Mode::Follower,
                    "leader"     => Mode::Leader,
                    "standalone" => Mode::Standalone,
                    _ => Mode::Unknown,
                };

                let znode = ZNode {
                    id: id,
                    mode: mode
                };

                Some(znode)
            },
            _ => None
        };

        znode
    }

    fn get_brokers(&self) -> Option<KafkaCluster> {
        let brokers = self.call_nc(&"wchc".to_string(), &"/brokers/ids".to_string());
        let topics = self.call_nc(&"wchc".to_string(), &"/brokers/topics".to_string());
        Some(KafkaCluster {
            ids: brokers,
            topics: topics,
        })
    }

}

struct ZkEnsembleService {
    nodes: Vec<(String, String)>
}

impl ZkEnsembleService {
    
    fn new(nodes: Vec<(String, String)>) -> ZkEnsembleService {
        ZkEnsembleService {
            nodes: nodes,
        }
    }
}

/// Splits a String of format `host:port` into a tuple.
fn split(s: &String) -> (String, String) {
    let pair_vec: Vec<String> = s.split(':').map(|s| s.to_string()).collect();
    
    if pair_vec.len() == 2 {
        (pair_vec[0].clone(), pair_vec[1].clone()) //probably better not to clone
    } else {
        panic!("Wrong parameter format. Should be 'host:port'");
    }
}

/// Returns length of the longest String in this Vector.
fn max_len(v: &Vec<&String>) -> usize {
    let max_host = v.iter().fold(v[0], |acc, &t| {
        if t.len() > acc.len() {
            t
        } else {
            acc
        }
    });

    return max_host.len();
}

fn main() {
    println!("Zookeeper ensemble status:");

    let args: Vec<String> = env::args().collect::<Vec<String>>().drain(1..).collect(); //drop the first arg
    let args_iter = args.iter();
    let args_split: Vec<(String, String)> = args_iter.map(|arg| split(&arg.to_string())).collect();
    //let hosts: Vec<&String> = args_split.iter().map(|arg| &arg.0)).collect();
    let hosts: Vec<&String> = args.iter().map(|arg| arg).collect();
    let max_host_len = max_len(&hosts);

    let hosts_size: usize = hosts.len();

    let (tx, rx) = mpsc::channel();

    let mut threads = vec![];

    let mut status = HashMap::new();
    let mut cluster = None;

    for (host, port) in &args_split {

        let txc = mpsc::Sender::clone(&tx);

        let h = host.clone();
        let p = port.clone();

        threads.push(thread::spawn(move || {
            let p2 = p.clone();
            let client = ZookeeperClient::new(h.clone(), p);
            let znode = client.get_status();
            let mode = znode.as_ref().map(|zn| zn.mode);
            if let Some(m) = mode {
                match m {
                    Mode::Leader => {
                        let c = client.get_brokers();
                        //println!("{:?}", c.map(|cl| cl.ids.clone()));
                        txc.send((format!("{}:{}", h, p2), znode, c));
                    },
                    _ => {
                        txc.send((format!("{}:{}", h, p2), znode, None));
                    },
                }
            } else {
                txc.send((format!("{}:{}", h, p2), znode, None));
            }
        }));

    }

    for (h, znode, cl) in rx {        
        status.insert(h, znode);
        println!("{:?}", cl.is_some());
        if let Some(c) = cl {
            println!("{}", "inside");
            println!("{:?}", &c.ids);
            cluster = Some(c);
        }

        let i = status.len();
        println!("len {}", i);
        println!("{}", hosts_size);
        if  i == hosts_size {
            break;
        }
    }

    for thread in threads {
        let _ = thread.join();
    }

    //keep the hosts ordering from the original parameter list
    for h in hosts {
        let znode = status.get(h.as_str()).unwrap();
        let (id, mode) = znode.as_ref().map_or_else(|| ("_".to_string(), "no connection".to_string()), |zn| (zn.id.clone(), zn.mode.to_string()));
        let color = znode.as_ref().map_or_else(|| Color::Blue, |zn| match zn.mode {
            Mode::Follower => Color::Cyan,
            Mode::Leader   => Color::Magenta,
            Mode::Standalone => Color::Yellow,
            Mode::Unknown  => Color::Red,
        });
        let styled_id = style(id)
            .with(Color::Yellow)
            .attribute(Attribute::Bold);
        let styled_mode = style(mode)
            .with(color)
            .attribute(Attribute::Bold);
        println!("{}", format!("{}: {:width$} : {}", styled_id, h, styled_mode, width = max_host_len));
    }

    let (ids, topics) = cluster.map(|c| (c.ids, c.topics)).unwrap_or((vec!(), vec!()));
    
    println!("\nKafka brokers:");
    println!("{}", format!("{:?}", ids));

    println!("\nKafka topics:");
    println!("{}", format!("{:?}", topics));
}
