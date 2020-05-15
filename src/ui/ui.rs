use std::io;

use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    Terminal,
    widgets::{Axis, BarChart, Block, Borders, Chart, Dataset, Marker, Paragraph, SelectableList, Tabs, Text, Widget},
};

use crate::App;
use crate::ui::app::{SlickTab, TabKind, ZMXTab};
use crate::jmx_client::model::HikariMetrics;
use crate::zio::model::FiberCount;

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
            TabKind::ZMX => &app.zmx.as_ref().map(|t| draw_zio_tab(&mut f, t, chunks[1])),
            TabKind::Slick => &app.slick.as_ref().map(|t| draw_slick_tab(&mut f, t, chunks[1]))
        };
    })
}

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
    where B: Backend,
{
    let text = [
        Text::raw("Contact us: "),
        Text::styled("info@scalac.io", Style::default().fg(Color::Blue)),
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

    draw_database_graphs(f, slick, chunks[0]);
    draw_text(f, chunks[1]);
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
            draw_hikari_graphs(f, db, chunks[1]);
        }
    }
}

fn draw_slick_graphs<B>(f: &mut Frame<B>, db: &SlickTab, area: Rect)
    where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let slick_threads_barchart: Vec<(&str, u64)> = db.slick_metrics.iter()
        .map(|x| ("", x.active_threads as u64))
        .collect();
    let active_threads = db.slick_metrics.back().map_or(0, |x| x.active_threads);
    BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title_style(Style::default().fg(Color::Cyan))
            .title(format!("Slick active threads: {} (max: {})", active_threads, db.slick_config.max_threads).as_ref()))
        .data(&slick_threads_barchart)
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

    let slick_queue_barchart: Vec<(&str, u64)> = db.slick_metrics.iter()
        .map(|x| ("", x.queue_size as u64))
        .collect();
    let queue_size = db.slick_metrics.back().map_or(0, |x| x.queue_size);
    BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title_style(Style::default().fg(Color::Cyan))
            .title(format!("Slick queue size: {} (max: {})", queue_size, db.slick_config.max_queue_size).as_ref()))
        .data(&slick_queue_barchart)
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

fn hikari_chart<F>(db: &SlickTab, f: F) -> Vec<(f64, f64)>
    where F: Fn(&HikariMetrics) -> i32, {
    db.hikari_metrics.iter().enumerate()
        .map(|(i, x)| (i as f64, f(x) as f64))
        .collect()
}

fn draw_hikari_graphs<B>(f: &mut Frame<B>, db: &SlickTab, area: Rect)
    where B: Backend,
{
    let total_chart: Vec<(f64, f64)> = hikari_chart(db, |x| x.total);
    let active_chart: Vec<(f64, f64)> = hikari_chart(db, |x| x.active);
    let idle_chart: Vec<(f64, f64)> = hikari_chart(db, |x| x.idle);
    let waiting_chart: Vec<(f64, f64)> = hikari_chart(db, |x| x.waiting);

    let datasets = [
        Dataset::default()
            .name("total")
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Blue))
            .data(&total_chart),
        Dataset::default()
            .name("active")
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Red))
            .data(&active_chart),
        Dataset::default()
            .name("waiting")
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .data(&waiting_chart),
        Dataset::default()
            .name("idle")
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Green))
            .data(&idle_chart)
    ];

    let max_connections = db.hikari_metrics.back().map_or(99, |x| x.total);
    let total_connections = db.hikari_metrics.back().map_or(0, |x| x.total);
    let active_connections = db.hikari_metrics.back().map_or(0, |x| x.active);
    let waiting_connections = db.hikari_metrics.back().map_or(0, |x| x.waiting);
    let idle_connections = db.hikari_metrics.back().map_or(0, |x| x.idle);

    Chart::default()
        .block(
            Block::default()
                .title(&format!(
                    "HikariCP (total={}, active={}, idle={}, waiting={})",
                    total_connections,
                    active_connections,
                    idle_connections,
                    waiting_connections
                ))
                .title_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels_style(Style::default().modifier(Modifier::ITALIC))
                .bounds([0.0, SlickTab::MAX_HIKARI_MEASURES as f64])
                .labels(&["older", "recent"])
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels_style(Style::default().modifier(Modifier::ITALIC))
                .bounds([-1.0, (max_connections + 1) as f64])
                .labels(&["0".to_owned(), ((max_connections as f64) / 2.0).to_string(), max_connections.to_string()])
        )
        .datasets(&datasets)
        .render(f, area);
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

fn fiber_count_chart<F>(db: &ZMXTab, f: F) -> Vec<(f64, f64)>
    where F: Fn(&FiberCount) -> i32, {
    db.fiber_counts.iter().enumerate()
        .map(|(i, x)| (i as f64, f(x) as f64))
        .collect()
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
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
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

                let running_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.running);
                let done_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.done);
                let finishing_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.finishing);
                let suspended_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.suspended);

                let datasets = [
                    Dataset::default()
                        .name("running")
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::Green))
                        .data(&running_chart),
                    Dataset::default()
                        .name("done")
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::LightBlue))
                        .data(&done_chart),
                    Dataset::default()
                        .name("finishing")
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::White))
                        .data(&finishing_chart),
                    Dataset::default()
                        .name("suspended")
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::Yellow))
                        .data(&suspended_chart)
                ];

                let max_fibers = zmx.fiber_counts.iter().map(|x| x.total()).max().unwrap_or(0);
                let total_fibers = zmx.fiber_counts.back().map_or(0, |x| x.total());
                let running_fibers = zmx.fiber_counts.back().map_or(0, |x| x.running);
                let done_fibers = zmx.fiber_counts.back().map_or(0, |x| x.done);
                let finishing_fibers = zmx.fiber_counts.back().map_or(0, |x| x.finishing);
                let suspended_fibers = zmx.fiber_counts.back().map_or(0, |x| x.suspended);

                Chart::default()
                    .block(
                        Block::default()
                            .title(&format!(
                                "Fibers (total={}, running={}, done={}, finishing={}, suspended={})",
                                total_fibers,
                                running_fibers,
                                done_fibers,
                                finishing_fibers,
                                suspended_fibers
                            ))
                            .title_style(Style::default().fg(Color::Cyan))
                            .borders(Borders::ALL)
                    )
                    .x_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([0.0, ZMXTab::MAX_FIBER_COUNT_MEASURES as f64])
                            .labels(&["older", "recent"])
                    )
                    .y_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .labels_style(Style::default().modifier(Modifier::ITALIC))
                            .bounds([-1.0, (max_fibers + 1) as f64])
                            .labels(&["0".to_owned(), ((max_fibers as f64) / 2.0).to_string(), max_fibers.to_string()])
                    )
                    .datasets(&datasets)
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
