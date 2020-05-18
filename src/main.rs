mod ui;
mod zio;
mod jmx_client;

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{stdout, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
    env,
};
use structopt::StructOpt;
use tui::{
    Terminal,
    backend::CrosstermBackend,
};

use ui::app::App;
use jmx_client::model::JMXConnectionSettings;

enum Event<I> {
    Input(I),
    Tick,
}

/// At least one of the following option sets has to be specified for panopticon-tui to launch:
///
/// - zio-zmx
///
/// - jmx + db-pool-name
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

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(cli.tick_rate);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = App::new(
        "PANOPTICON-TUI",
        cli.zio_zmx.clone(),
        cli.jmx_settings(),
    );

    if let Err(err) = app.initialize_connections() {
        app.quit(Some(err));
    } else {
        terminal.clear()?;
    }

    loop {
        ui::ui::draw(&mut terminal, &mut app)?;
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    break;
                }                
                KeyCode::Char(c) => app.on_key(c),
                KeyCode::Enter => app.on_enter(),
                KeyCode::Left => app.on_left(),
                KeyCode::Up => app.on_up(),
                KeyCode::Right => app.on_right(),
                KeyCode::Down => app.on_down(),
                KeyCode::PageUp => app.on_page_up(),
                KeyCode::PageDown => app.on_page_down(),
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            if let Some(message) = app.exit_reason {
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                println!("{}", message);
            }
            break;
        }
    }

    Ok(())
}
