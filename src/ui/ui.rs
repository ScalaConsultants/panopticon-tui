use std::io;

use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    Terminal,
    widgets::{BarChart, Block, Borders, Paragraph, SelectableList, Tabs, Text, Widget},
};

use crate::App;
use crate::ui::app::{SlickTab, TabKind, ZMXTab};

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        Tabs::default()
            .block(Block::default()
                .borders(Borders::ALL)
                .title_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
                .title(app.title))
            .titles(&app.tabs.titles())
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);
        match app.tabs.current().kind {
            TabKind::ZMX => draw_zio_tab(&mut f, &app.zmx, chunks[1]),
            TabKind::Slick => draw_slick_tab(&mut f, &app.slick.as_ref().unwrap(), chunks[1]),
        };
    })
}

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
    where B: Backend,
{
    let text = [
        Text::raw("Contact us: "),
        Text::styled("zio@scalac.io", Style::default().fg(Color::Blue)),
    ];
    Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("by Scalac Sp.z o.o.")
                .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::BOLD)),
        )
        .wrap(true)
        .render(f, area);
}

fn draw_slick_tab<B>(f: &mut Frame<B>, slick: &SlickTab, area: Rect)
    where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Min(7), Constraint::Length(3)].as_ref())
        .split(area);

    match &slick.slick_error {
        None => { draw_database_graphs(f, slick, chunks[0]) }
        Some(err) => { draw_db_connection_error(f, err, chunks[0]) }
    };

    draw_text(f, chunks[1]);
}

fn draw_db_connection_error<B>(f: &mut Frame<B>, err: &String, area: Rect)
    where B: Backend,
{
    Paragraph::new([Text::raw(format!("Error: {}", err))].iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Database JMX metrics")
                .title_style(Style::default().fg(Color::Cyan))
        )
        .render(f, area);
}


fn draw_database_graphs<B>(f: &mut Frame<B>, db: &SlickTab, area: Rect)
    where B: Backend,
{
    let constraints: Vec<Constraint> = if db.has_hikari {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let chunks = Layout::default()
        .constraints(constraints.as_ref())
        .direction(Direction::Horizontal)
        .split(area);
    {
        draw_slick_graphs(f, db, chunks[0]);
        if db.has_hikari {
            draw_slick_graphs(f, db, chunks[1]);
        }
    }
}

fn draw_slick_graphs<B>(f: &mut Frame<B>, db: &SlickTab, area: Rect)
    where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let active_threads = db.slick_metrics.back().map_or(0, |x| x.active_threads);
    BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title_style(Style::default().fg(Color::Cyan))
            .title(format!("Slick active threads: {} (max: {})", active_threads, db.slick_config.max_threads).as_ref()))
        .data(&db.slick_threads_barchart)
        .max(db.slick_config.max_threads as u64)
        .bar_width(3)
        .bar_gap(1)
        .value_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
        )
        .style(Style::default().fg(Color::Green))
        .render(f, chunks[0]);

    let queue_size = db.slick_metrics.back().map_or(0, |x| x.queue_size);
    BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title_style(Style::default().fg(Color::Cyan))
            .title(format!("Slick queue size: {} (max: {})", queue_size, db.slick_config.max_queue_size).as_ref()))
        .data(&db.slick_queue_barchart)
        .max(db.slick_config.max_queue_size as u64)
        .bar_width(3)
        .bar_gap(1)
        .value_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Blue)
        )
        .style(Style::default().fg(Color::Blue))
        .render(f, chunks[1]);
}

fn draw_zio_tab<B>(f: &mut Frame<B>, zmx: &ZMXTab, area: Rect)
    where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Min(7), Constraint::Length(3)].as_ref())
        .split(area);
    draw_fiber_list(f, zmx, chunks[0]);
    draw_text(f, chunks[1]);
}

fn draw_fiber_list<B>(f: &mut Frame<B>, zmx: &ZMXTab, area: Rect)
    where B: Backend,
{
    let constraints = vec![Constraint::Percentage(100)];
    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(chunks[0]);
        {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[0]);
            {
                let chunks = Layout::default()
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(chunks[0]);

                SelectableList::default()
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default().fg(Color::Cyan))
                        .title("Fibers (press <Enter> to take a snapshot)"))
                    .items(&zmx.fibers.items)
                    .select(Some(zmx.fibers.selected))
                    .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
                    .highlight_symbol(">")
                    .render(f, chunks[0]);

                BarChart::default()
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default().fg(Color::Cyan))
                        .title("Fiber count"))
                    .data(&zmx.barchart)
                    .bar_width(3)
                    .bar_gap(2)
                    .value_style(
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Green)
                            .modifier(Modifier::ITALIC),
                    )
                    .label_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().fg(Color::Green))
                    .render(f, chunks[1]);
            }
            let text = [Text::raw(zmx.selected_fiber_dump.0.to_owned())];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Fiber dump (press <PageUp>/<PageDown> to scroll)")
                        .title_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(true)
                .scroll(zmx.scroll)
                .render(f, chunks[1]);
        }
    }
}
