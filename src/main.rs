mod ui;
mod zio;

use crossterm::{
    input::{input, InputEvent, KeyEvent},
    screen::AlternateScreen,
};
use std::{
    io::stdout,
    sync::mpsc,
    thread,
    time::Duration,
};
use structopt::StructOpt;
use tui::{
    Terminal,
    backend::CrosstermBackend, 
};

use ui::app::App;

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(long = "tick-rate", default_value = "2000")]
    tick_rate: u64,
    #[structopt(long = "zio-zmx")]
    zio_zmx: Option<String>,
}

fn main() -> Result<(), failure::Error> {
    let cli = Cli::from_args();

    let zio_zmx_addr = cli.zio_zmx.to_owned().map(|x| x.clone()).expect("No ZIO-ZMX address.");

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
            thread::sleep(Duration::from_millis(cli.tick_rate));
        }
    });

    let mut app = App::new(
        "ZIO-ZMX-TUI", 
        zio_zmx_addr,
        vec![],
        vec![],
    );

    terminal.clear()?;

    loop {
        ui::ui::draw(&mut terminal, &app)?;
        match rx.recv()? {
            Event::Input(event) => match event {
                KeyEvent::Char(c) => app.on_key(c),
                KeyEvent::Enter => app.on_enter(),
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
