mod ui;
mod zio;
mod jmx_client;
mod akka_actor_tree;

use crossterm::{
    input::{input, InputEvent, KeyEvent},
    screen::AlternateScreen,
};
use std::{
    io::stdout,
    sync::mpsc,
    thread,
    time::Duration,
    env,
};
use structopt::StructOpt;
use tui::{
    Terminal,
    backend::CrosstermBackend,
};
use ui::app::App;
use ui::fetcher::{Fetcher, FetcherRequest, FetcherResponse};
use jmx_client::model::JMXConnectionSettings;
use crate::akka_actor_tree::model::{AkkaActorTreeSettings};
use crate::ui::app::TabKind;

enum Event<I> {
    Input(I),
    Tick,
    FetcherResponse(FetcherResponse),
}

/// At least one of the following option sets has to be specified for panopticon-tui to launch:
///
/// - zio-zmx
///
/// - jmx + db-pool-name
///
/// - actor-tree
#[derive(Debug, StructOpt)]
struct Cli {
    /// Frequency (in ms) to use for fetching metrics.
    /// Don't set this too low, because currently zmx tab does a full fiber dump every tick
    #[structopt(long = "tick-rate", default_value = "2000")]
    tick_rate: u64,
    /// Address of zio-zmx server, e.g. localhost:6789
    #[structopt(long = "zio-zmx")]
    zio_zmx: Option<String>,
    /// Address of remote jmx source, e.g. localhost:9010
    #[structopt(long = "jmx")]
    jmx: Option<String>,
    /// Optional username for authorized jmx access
    #[structopt(long = "jmx-username")]
    jmx_username: Option<String>,
    /// Optional password for authorized jmx access
    #[structopt(long = "jmx-password")]
    jmx_password: Option<String>,
    /// Connection pool name, used to qualify JMX beans for Slick and/or HikariCP
    #[structopt(long = "db-pool-name")]
    db_pool_name: Option<String>,
    /// Address of http endpoint to get akka actor tree
    #[structopt(long = "actor-tree")]
    actor_tree: Option<String>,
    /// Address of http endpoint to get current actor count
    #[structopt(long = "actor-count")]
    actor_count: Option<String>,
    /// Time period (in ms) to assemble akka actor tree
    #[structopt(long = "actor-tree-timeout", default_value = "1000")]
    actor_tree_timeout: u64,
}

impl Cli {
    fn jmx_settings(&self) -> Option<JMXConnectionSettings> {
        match (&self.jmx, &self.db_pool_name) {
            (Some(addr), Some(db_pool)) => Some(JMXConnectionSettings {
                address: addr.clone(),
                username: self.jmx_username.clone(),
                password: self.jmx_password.clone(),
                db_pool_name: db_pool.clone(),
            }),
            _ => None
        }
    }

    fn akka_actor_tree_settings(&self) -> Option<AkkaActorTreeSettings> {
        match (&self.actor_tree, &self.actor_count) {
            (Some(tree_addr), Some(count_addr)) => Some(AkkaActorTreeSettings {
                tree_address: tree_addr.to_owned(),
                tree_timeout: self.actor_tree_timeout,
                count_address: count_addr.to_owned(),
                count_timeout: (self.tick_rate as f64 * 0.8) as u64,
            }),
            _ => None
        }
    }
}

fn main() -> Result<(), failure::Error> {
    let cli = Cli::from_args();

    // disable jmx crate logging
    env::set_var("J4RS_CONSOLE_LOG_LEVEL", "disabled");

    if cli.zio_zmx.is_none() && cli.jmx_settings().is_none() {
        let mut clap = Cli::clap();
        println!("Nothing to monitor. Please check the following help message.\n");
        clap.print_long_help().expect("Failed printing help message");
        return Ok(());
    }

    let tick_rate = cli.tick_rate;
    let has_jmx = cli.jmx_settings().is_some();

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

    let mut app = App::new(
        "ZIO-ZMX-TUI",
        cli.zio_zmx.clone(),
        cli.jmx_settings(),
        cli.akka_actor_tree_settings(),
    );

    terminal.clear()?;

    // Setup fetcher interaction
    let (txf, rxf) = mpsc::channel();
    {
        let tx = tx.clone();
        thread::spawn(move || {
            let respond = |r| tx.send(Event::FetcherResponse(r)).unwrap();

            match Fetcher::new(cli.zio_zmx.clone(),
                               cli.jmx_settings(),
                               cli.akka_actor_tree_settings()) {
                Err(e) => {
                    eprintln!("Responding with failure {}", e);
                    loop {
                        rxf.recv().unwrap();
                        respond(FetcherResponse::FatalFailure(e.to_owned()))
                    }
                }
                Ok(fetcher) =>
                    loop {
                        match rxf.recv().unwrap() {
                            FetcherRequest::FiberDump =>
                                respond(FetcherResponse::FiberDump(fetcher.dump_fibers())),
                            FetcherRequest::RegularFiberDump =>
                                respond(FetcherResponse::RegularFiberDump(fetcher.dump_fibers())),
                            FetcherRequest::HikariMetrics =>
                                respond(FetcherResponse::HikariMetrics(fetcher.get_hikari_metrics())),
                            FetcherRequest::SlickMetrics =>
                                respond(FetcherResponse::SlickMetrics(fetcher.get_slick_metrics())),
                            FetcherRequest::SlickConfig =>
                                respond(FetcherResponse::SlickConfig(fetcher.get_slick_config())),
                            FetcherRequest::ActorTree =>
                                respond(FetcherResponse::ActorTree(fetcher.get_actor_tree())),
                            FetcherRequest::ActorCount =>
                                respond(FetcherResponse::ActorCount(fetcher.get_actor_count())),
                        }
                    }
            }
        });
    }

    {
        let tx = tx.clone();
        let txf = txf.clone();
        thread::spawn(move || {
            if has_jmx {
                txf.send(FetcherRequest::SlickConfig).unwrap();
                txf.send(FetcherRequest::HikariMetrics).unwrap();
                txf.send(FetcherRequest::SlickMetrics).unwrap();
            }
            loop {
                tx.send(Event::Tick).unwrap();
                thread::sleep(Duration::from_millis(tick_rate));
            }
        });
    }

    loop {
        if !app.should_quit {
            ui::ui::draw(&mut terminal, &app)?;
        }

        match rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent::Char(c) => app.on_key(c),
                KeyEvent::Left => app.on_left(),
                KeyEvent::Up => app.on_up(),
                KeyEvent::Right => app.on_right(),
                KeyEvent::Down => app.on_down(),
                KeyEvent::PageUp => app.on_page_up(),
                KeyEvent::PageDown => app.on_page_down(),
                KeyEvent::Enter => {
                    match app.tabs.current().kind {
                        TabKind::ZMX => txf.send(FetcherRequest::FiberDump)?,
                        TabKind::Slick => {}
                        TabKind::AkkaActorTree => txf.send(FetcherRequest::ActorTree)?,
                    }
                }
                _ => {}
            },
            Event::FetcherResponse(r) => match r {
                FetcherResponse::FatalFailure(e) =>
                    app.quit(Some(e)),

                FetcherResponse::FiberDump(d) =>
                    match d {
                        Err(e) => app.quit(Some(e)),
                        Ok(x) => app.zmx.as_mut().unwrap().replace_fiber_dump(x),
                    },
                FetcherResponse::RegularFiberDump(d) =>
                    match d {
                        Err(e) => app.quit(Some(e)),
                        Ok(x) => app.zmx.as_mut().unwrap().append_fiber_dump_for_counts(x),
                    },
                FetcherResponse::HikariMetrics(d) =>
                    match d {
                        Err(_) => app.slick.as_mut().unwrap().has_hikari = false,
                        Ok(x) => {
                            app.slick.as_mut().unwrap().has_hikari = true;
                            app.slick.as_mut().unwrap().append_hikari_metrics(x)
                        }
                    },
                FetcherResponse::SlickMetrics(d) =>
                    match d {
                        Err(e) => app.quit(Some(e)),
                        Ok(x) => app.slick.as_mut().unwrap().append_slick_metrics(x)
                    },
                FetcherResponse::SlickConfig(d) =>
                    match d {
                        Err(e) => app.quit(Some(e)),
                        Ok(x) => app.slick.as_mut().unwrap().replace_slick_config(x)
                    },
                FetcherResponse::ActorTree(d) =>
                    match d {
                        Err(e) => app.quit(Some(e)),
                        Ok(x) => app.actor_tree.as_mut().unwrap().update_actor_tree(x)
                    },
                FetcherResponse::ActorCount(d) =>
                    match d {
                        Err(e) => app.quit(Some(e)),
                        Ok(x) => app.actor_tree.as_mut().unwrap().append_actor_count(x)
                    },
            }

            Event::Tick => {
                if app.zmx.is_some() {
                    txf.send(FetcherRequest::RegularFiberDump)?;
                }

                match &app.slick {
                    Some(s) => {
                        txf.send(FetcherRequest::SlickMetrics)?;
                        if s.has_hikari {
                            txf.send(FetcherRequest::HikariMetrics)?;
                        }
                    }
                    None => {}
                }

                if app.actor_tree.is_some() {
                    txf.send(FetcherRequest::ActorCount)?;
                }
            }
        }
        if app.should_quit {
            break;
        }
    }

    &terminal.backend().alternate_screen().unwrap().to_main().unwrap();
    app.exit_reason.map(|e| println!("{}", e));
    Ok(())
}
