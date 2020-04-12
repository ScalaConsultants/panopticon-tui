
mod app;
mod ui;
mod zio;
mod zookeeper;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::mpsc;

use crossterm::{
    input::{input, InputEvent, KeyEvent},
    screen::AlternateScreen,
};
use std::{
    io::stdout,
    thread,
    time::Duration,
};
use tui::{
    backend::CrosstermBackend, 
    Terminal,
};

use app::App;
use zio::model::Fiber;
use zookeeper::model::{Mode, ZAddr};

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

enum Event<I> {
    Input(I),
    Tick,
}

struct FiberDump {
    pub ids: Vec<Fiber>,
    pub dumps: Vec<String>,
}

struct ZookeeperStatus {
    nodes: Vec<String>,
    wchc_all: Vec<Vec<String>>,
}

fn get_fiber_dump() -> Result<FiberDump, Box<dyn Error>> {
    zio::zmx_client::get_dump().map(|a| {
        let mut fiber_ids: Vec<Fiber> = vec![];
        let mut fiber_dumps: Vec<String> = vec![];
        for (id, dump) in a {
            fiber_ids.push(id);
            fiber_dumps.push(dump);
        }
    
        FiberDump {
            ids: fiber_ids,
            dumps: fiber_dumps,
        }
    })
}

fn get_zookeeper_status(hosts: Vec<&String>) -> ZookeeperStatus {

    let hosts_and_ports: Vec<(String, String)> = hosts.iter().map(|arg| split(&arg.to_string())).collect();

    let max_host_len = max_len(&hosts);

    let hosts_size: usize = hosts.len();

    let (tx, rx) = mpsc::channel();

    let mut threads = vec![];

    let mut znode_status = HashMap::new();
    let mut znode_wchc   = HashMap::new();

    for (host, port) in &hosts_and_ports {

        let txc = mpsc::Sender::clone(&tx);

        let h = host.clone();
        let p = port.clone();

        threads.push(thread::spawn(move || {
            let p2 = p.clone();
            //let client = ZookeeperClient::new(h.clone(), p);
            let znode = zookeeper::zookeeper_client::get_status(ZAddr { host: h.clone(), port: p.clone() });
            let mode = znode.as_ref().map(|zn| zn.mode);
            if let Some(m) = mode {
                match m {
                    Mode::Leader | Mode::Follower | Mode::Standalone => {
                        let c = zookeeper::zookeeper_client::get_wchc(ZAddr { host: h.clone(), port: p.clone() });
                        //println!("{:?}", c.map(|cl| cl.ids.clone()));
                        let _ = txc.send((format!("{}:{}", h, p2), znode, c));
                    },
                    _ => {
                        let _ = txc.send((format!("{}:{}", h, p2), znode, vec![]));
                    },
                }
            } else {
                let _ = txc.send((format!("{}:{}", h, p2), znode, vec![]));
            }
        }));

    }

    for (h, znode, cl) in rx {        
        znode_status.insert(h.clone(), znode);
        znode_wchc.insert(h, cl);

        let i = znode_status.len();
        if  i == hosts_size {
            break;
        }
    }

    for thread in threads {
        let _ = thread.join();
    }

    let mut zookeeper_nodes: Vec<String> = vec![];
    let mut zookeeper_wchc_all: Vec<Vec<String>> = vec![];
    //keep the hosts ordering from the original parameter list
    for h in hosts {
        let znode = znode_status.get(h.as_str()).unwrap();
        let zwchc = znode_wchc.get(h.as_str()).unwrap();
        println!("{:?}", &zwchc);
        let (id, mode) = znode.as_ref().map_or_else(|| ("_".to_string(), "no connection".to_string()), |zn| (zn.id.clone(), zn.mode.to_string()));
        let s: String = format!("{}: {:width$} : {}", id, h, mode, width = max_host_len);
        zookeeper_nodes.push(s.clone());
        zookeeper_wchc_all.push(zwchc.to_vec());
        println!("aaa {:?}", &zookeeper_wchc_all);
        println!("{}", s);
    }

    ZookeeperStatus {
        nodes: zookeeper_nodes,
        wchc_all: zookeeper_wchc_all,
    }
}

fn main() -> Result<(), failure::Error> {
    let tick_rate: u64 = 2000;

    let args: Vec<String> = env::args().collect::<Vec<String>>().drain(1..).collect(); //drop the first arg
    let zk_hosts: Vec<&String> = args.iter().map(|arg| arg).collect();

    let screen = AlternateScreen::to_alternate(true)?;
    let backend = CrosstermBackend::with_alternate_screen(stdout(), screen)?;
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();
    {
        let tx = tx.clone();
        thread::spawn(move || {
            let input = input();
            let mut reader = input.read_sync();
            loop {
                match reader.next() {
                    Some(InputEvent::Keyboard(key)) => {
                        if let Err(_) = tx.send(Event::Input(key.clone())) {
                            return;
                        }
                        if key == KeyEvent::Char('q') {
                            return;
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    thread::spawn(move || {
        let tx = tx.clone();
        loop {
            tx.send(Event::Tick).unwrap();
            thread::sleep(Duration::from_millis(tick_rate));
        }
    });

    let fd = get_fiber_dump().unwrap();//TODO take care of error
    let zs = get_zookeeper_status(zk_hosts);

    let fd_fmt: Vec<String> = fd.ids.iter().map(|f| f.to_string()).collect();

    let mut app = App::new(
        "Panopticon", 
        zs.nodes.iter().map(|z| z.as_str()).collect(), 
        zs.wchc_all.iter().map(|v| v.iter().map(|z| z.as_str()).collect()).collect(),
        fd_fmt.iter().map(|f| f.as_str()).collect(),
        fd.dumps.iter().map(|z| z.as_str()).collect()
    );

    terminal.clear()?;

    loop {
        ui::draw(&mut terminal, &app)?;
        match rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent::Char(c) => app.on_key(c),
                KeyEvent::Left => app.on_left(),
                KeyEvent::Up => app.on_up(),
                KeyEvent::Right => app.on_right(),
                KeyEvent::Down => app.on_down(),
                KeyEvent::PageUp => app.on_page_up(),
                KeyEvent::PageDown => app.on_page_down(),
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
