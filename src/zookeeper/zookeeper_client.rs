
use std::str;
use std::process::Command;
use crate::zookeeper::model::*;

//This method sends a 4 Letter Word copmmnd to the Zookeeper node using netcat.
//Possible errors include:
//- netcat is not available in the system
//- Zookeeper node is down or not available on this address
//- Zookeeper node is up but in a state where it doesn't answer requests (eg. ensemble is below quorum limit)
//- Zookeeper node doesn't serve this particular word (the 4LWord is not whitelisted)
//
fn call_nc(addr: &ZAddr, word: Word) -> Vec<String> {
    let command = match word {
        Word::Srvr => "srvr",
        Word::Conf => "conf",
        Word::Wchc => "wchc",
    };

    let com = format!("echo -n '{}' | nc -w 3 {} {}", command, addr.host, addr.port);
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(com)
        .output()
        .expect("no connection");
    
    let output_status = output.status;
    
    if output_status.success() {
        let output_std: Vec<u8> = output.stdout.clone();

        let output_str = str::from_utf8(&output_std).unwrap();
        let output_lines = output_str.lines().map(|x| x.trim().to_string()).collect();
            
        println!("{:?}", output_lines);
        return output_lines;
    } else {
        return vec![];
    }
}

fn grep(lines: Vec<String>, contains: String) -> Vec<String> {
    lines.iter().filter(|x| x.starts_with(&contains)).map(|x| x.clone()).collect()
}

fn drop_n_chars(s: String, n: usize) -> String {
    match s.char_indices().nth(n) {
        Some((n, _)) => {
            s.to_string().drain(n..).collect::<String>()
        },
        None         => "".to_string()
    }
}

pub fn get_status(addr: ZAddr) -> Option<ZNode> {
    println!("get_status");
    let srvr_out = call_nc(&addr, Word::Srvr);
    let conf_out = call_nc(&addr, Word::Conf);

    let mut modes = grep(srvr_out, "Mode".to_string());
    let mut server_ids = grep(conf_out, "serverId".to_string());

    let mode = modes.pop().map(|x| drop_n_chars(x, "Mode".len() + ": ".len()));
    let server_id = server_ids.pop().map(|x| drop_n_chars(x, "serverId".len() + "=".len()));

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

// pub fn get_brokers(addr: ZAddr) -> Option<KafkaCluster> {
//     let brokers_out = call_nc(&addr, Word::Wchc);
//     let topics_out = call_nc(&addr, Word::Wchc);

//     let brokers = grep(brokers_out, "/brokers/ids".to_string());
//     let topics = grep(topics_out, "/brokers/topics".to_string());
//     Some(KafkaCluster {
//         ids: brokers,
//         topics: topics,
//     })
// }

pub fn get_wchc(addr: ZAddr) -> Vec<String> {
    println!("get_wchc");
    call_nc(&addr, Word::Wchc)
}
