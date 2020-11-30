use std::io;

use chrono::prelude::*;
use humantime::format_duration;
use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    Terminal,
    widgets::{Axis, BarChart, Block, Borders, Chart, Dataset, List, Paragraph, Tabs},
};
use tui::text::Span;
use tui::widgets::{ListItem, Wrap};

use crate::akka::model::DeadLettersWindow;
use crate::app::{AkkaTab, App, AppTabKind, SlickTab, ZMXTab};
use crate::jmx::model::HikariMetrics;
use crate::zio::model::FiberCount;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        let tabs = app.tabs.to_owned();
        let titles = tabs.titles();
        let tabs_widget = Tabs::new(titles)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(app.title, Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))))
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(tabs.index);
        f.render_widget(tabs_widget, chunks[0]);
        match tabs.current().kind {
            AppTabKind::ZMX => &app.zmx.as_mut().map(|mut t| draw_zio_tab(&mut f, &mut t, chunks[1])),
            AppTabKind::Slick => &app.slick.as_ref().map(|t| draw_slick_tab(&mut f, t, chunks[1])),
            AppTabKind::Akka => &app.akka.as_mut().map(|t| draw_akka_tab(&mut f, t, chunks[1])),
        };
    })
}

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
    where B: Backend,
{
    let p = Paragraph::new("")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("by Scalac", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
        )
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
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
    let active_threads_title = format!("Slick active threads: {} (max: {})", active_threads, db.slick_config.max_threads);
    let active_threads_bc = BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&active_threads_title, Style::default().fg(Color::Cyan))))
        .data(&slick_threads_barchart)
        .max(db.slick_config.max_threads as u64)
        .bar_width(3)
        .bar_gap(1)
        .value_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
        )
        .style(Style::default().fg(Color::Green));
    f.render_widget(active_threads_bc, chunks[0]);

    let slick_queue_data: Vec<(&str, u64)> = db.slick_metrics.iter()
        .map(|x| ("", x.queue_size as u64))
        .collect();
    let queue_size = db.slick_metrics.back().map_or(0, |x| x.queue_size);
    let queue_size_title = format!("Slick queue size: {} (max: {})", queue_size, db.slick_config.max_queue_size);
    let slick_queue_bc = BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&queue_size_title, Style::default().fg(Color::Cyan))))
        .data(&slick_queue_data)
        .max(db.slick_config.max_queue_size as u64)
        .bar_width(3)
        .bar_gap(1)
        .value_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Blue)
        )
        .style(Style::default().fg(Color::Blue));
    f.render_widget(slick_queue_bc, chunks[1]);
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

    let datasets = vec![
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

    let title = format!(
        "HikariCP (total={}, active={}, idle={}, waiting={})",
        total_connections,
        active_connections,
        idle_connections,
        waiting_connections
    );
    let label = vec!["0".to_owned(), ((max_connections as f64) / 2.0).to_string(), max_connections.to_string()];
    let c = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(&title, Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL)
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("older", Style::default().add_modifier(Modifier::ITALIC)),
                    Span::styled("recent", Style::default().add_modifier(Modifier::ITALIC))
                ])
                .bounds([0.0, SlickTab::MAX_HIKARI_MEASURES as f64])
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(label.into_iter().map(|l| Span::styled(l, Style::default().add_modifier(Modifier::ITALIC))).collect())
                .bounds([-1.0, (max_connections + 1) as f64])
        );
    f.render_widget(c, area);
}


fn draw_zio_tab<B>(f: &mut Frame<B>, zmx: &mut ZMXTab, area: Rect)
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

fn dead_letters_window_chart<F>(tab: &AkkaTab, f: F) -> Vec<(f64, f64)>
    where F: Fn(&DeadLettersWindow) -> u32, {
    tab.dead_letters_windows.iter().enumerate()
        .map(|(i, x)| (i as f64, f(x) as f64))
        .collect()
}

fn draw_fiber_list<B>(f: &mut Frame<B>, zmx: &mut ZMXTab, area: Rect)
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

                let items: Vec<ListItem<'_>> = zmx.fibers.items.iter().map(|i| ListItem::new(i.to_owned())).collect();

                let list = List::new(items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled("Fibers (press <Enter> to take a snapshot)", Style::default().fg(Color::Cyan))))
                    .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .highlight_symbol(">");
                f.render_stateful_widget(list, chunks[0], &mut zmx.fibers.state);

                let running_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.running);
                let done_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.done);
                let finishing_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.finishing);
                let suspended_chart: Vec<(f64, f64)> = fiber_count_chart(zmx, |x| x.suspended);

                let datasets = vec![
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

                let title = format!(
                    "Fibers (total={}, running={}, done={}, finishing={}, suspended={})",
                    total_fibers,
                    running_fibers,
                    done_fibers,
                    finishing_fibers,
                    suspended_fibers
                );
                let label = vec!["0".to_owned(), ((max_fibers as f64) / 2.0).to_string(), max_fibers.to_string()];
                let c = Chart::new(datasets)
                    .block(
                        Block::default()
                            .title(Span::styled(&title, Style::default().fg(Color::Cyan)))
                            .borders(Borders::ALL)
                    )
                    .x_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .labels(vec![
                                Span::styled("older", Style::default().add_modifier(Modifier::ITALIC)),
                                Span::styled("recent", Style::default().add_modifier(Modifier::ITALIC))
                            ])
                            .bounds([0.0, ZMXTab::MAX_FIBER_COUNT_MEASURES as f64])
                    )
                    .y_axis(
                        Axis::default()
                            .style(Style::default().fg(Color::Gray))
                            .labels(label.into_iter().map(|l| Span::styled(l, Style::default().add_modifier(Modifier::ITALIC))).collect())
                            .bounds([-1.0, (max_fibers + 1) as f64])
                    );
                f.render_widget(c, chunks[1]);
            }

            let p = Paragraph::new(zmx.selected_fiber_dump.0.to_owned())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled("Fiber dump (press <PageUp>/<PageDown> to scroll)", Style::default().fg(Color::Cyan)))
                )
                .wrap(Wrap { trim: true })
                .scroll((zmx.scroll, 0));
            f.render_widget(p, chunks[1]);
        }
    }
}

fn draw_akka_tab<B>(f: &mut Frame<B>, tab: &mut AkkaTab, area: Rect)
    where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Min(7), Constraint::Length(3)].as_ref())
        .split(area);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(chunks[0]);
        {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[0]);
            draw_actor_tree(f, tab, chunks[0]);
            draw_actor_count_chart(f, tab, chunks[1]);
        }
        {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[1]);
            draw_dead_letters_logs(f, tab, chunks[0]);
            draw_dead_letters_window_chart(f, tab, chunks[1]);
        }
    }
    draw_text(f, chunks[1]);
}

fn draw_dead_letters_logs<B>(f: &mut Frame<B>, tab: &mut AkkaTab, area: Rect)
    where B: Backend
{
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(6)].as_ref())
        .split(area);
    let titles = tab.dead_letters_tabs.titles();
    let tabs_widget = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Dead Letter logs (<A> left tab, <D> right tab)"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Blue))
        .select(tab.dead_letters_tabs.index);
    f.render_widget(tabs_widget, chunks[0]);
    draw_dead_letter_log(f, tab, chunks[1]);
    draw_dead_letter_message_details(f, tab, chunks[2]);
}

fn draw_dead_letter_log<B>(f: &mut Frame<B>, tab: &mut AkkaTab, area: Rect)
    where B: Backend {
    let items: Vec<ListItem<'_>> = tab.dead_letters_log.items.iter().map(|i| ListItem::new(i.summary())).collect();

    let title = format!("{:?}, {} total (↑↓ select message)", tab.dead_letters_tabs.current().kind, tab.dead_letters_log.items.len());
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&title, Style::default().fg(Color::Cyan))))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">");

    f.render_stateful_widget(list, area, &mut tab.dead_letters_log.state);
}

fn draw_dead_letter_message_details<B>(f: &mut Frame<B>, tab: &mut AkkaTab, area: Rect)
    where B: Backend {
    let text = match tab.dead_letters_log.selected() {
        None => "Select a message to see details".to_owned(),
        Some(m) => {
            let reason = match &m.reason {
                None => "".to_owned(),
                Some(r) => format!("Reason: {}\n", r)
            };
            // todo: format timestamp with system locale
            format!("Data: {}\nTimestamp: {}\nSender: {}\nReceiver: {}\n{}", m.message, m.readable_timestamp().format("%d.%m.%Y %H:%M:%S"), m.sender, m.recipient, reason)
        }
    };

    let p = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_dead_letters_window_chart<B>(f: &mut Frame<B>, tab: &mut AkkaTab, area: Rect)
    where B: Backend
{
    let dead_letters_chart: Vec<(f64, f64)> = dead_letters_window_chart(tab, |x| x.dead_letters.count);
    let unhandled_chart: Vec<(f64, f64)> = dead_letters_window_chart(tab, |x| x.unhandled.count);
    let dropped_chart: Vec<(f64, f64)> = dead_letters_window_chart(tab, |x| x.dropped.count);

    let datasets = vec![
        Dataset::default()
            .name("dead_letters")
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::Green))
            .data(&dead_letters_chart),
        Dataset::default()
            .name("unhandled")
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::LightBlue))
            .data(&unhandled_chart),
        Dataset::default()
            .name("dropped")
            .marker(Marker::Braille)
            .style(Style::default().fg(Color::White))
            .data(&dropped_chart),
    ];

    let max = tab.dead_letters_windows.iter().map(|x| x.max()).max().unwrap_or(0);
    let total = tab.dead_letters_windows.back().map_or(0, |x| x.total());
    let dead_letters = tab.dead_letters_windows.back().map_or(0, |x| x.dead_letters.count);
    let unhandled = tab.dead_letters_windows.back().map_or(0, |x| x.unhandled.count);
    let dropped = tab.dead_letters_windows.back().map_or(0, |x| x.dropped.count);

    let title = format!(
        "Dead Letters for last {}ms (total={}, dead letters={}, unhandled={}, dropped={})",
        tab.dead_letters_windows.back().map_or(0, |x| x.within_millis),
        total,
        dead_letters,
        unhandled,
        dropped
    );
    let label = vec!["0".to_owned(), ((max as f64) / 2.0).to_string(), max.to_string()];
    let c = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(&title, Style::default().fg(Color::Cyan)))
                .borders(Borders::ALL)
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("older", Style::default().add_modifier(Modifier::ITALIC)),
                    Span::styled("recent", Style::default().add_modifier(Modifier::ITALIC))
                ])
                .bounds([0.0, AkkaTab::MAX_DEAD_LETTERS_WINDOW_MEASURES as f64])
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .labels(label.into_iter().map(|l| Span::styled(l, Style::default().add_modifier(Modifier::ITALIC))).collect())
                .bounds([-1.0, (max + 1) as f64])
        );
    f.render_widget(c, area);
}

fn draw_actor_tree<B>(f: &mut Frame<B>, tab: &mut AkkaTab, area: Rect)
    where B: Backend,
{
    let items: Vec<ListItem<'_>> = tab.actors.items.iter().map(|i| ListItem::new(i.to_owned())).collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Actors (<Enter> to reload, <PageUp>/<PageDown> to scroll)", Style::default().fg(Color::Cyan))))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">");

    f.render_stateful_widget(list, area, &mut tab.actors.state);
}

fn draw_actor_count_chart<B>(f: &mut Frame<B>, tab: &AkkaTab, area: Rect)
    where B: Backend,
{
    let data: Vec<(&str, u64)> = tab.actor_counts.iter()
        .map(|x| ("", x.to_owned()))
        .collect();

    let title = format!(
        "Running actors: {}. System started {}, uptime = {}",
        tab.system_status.actor_count,
        NaiveDateTime::from_timestamp((tab.system_status.start_time / 1000) as i64, 0).format("%d.%m.%Y %H:%M:%S"),
        format_duration(std::time::Duration::from_secs(tab.system_status.uptime))
    );
    let count_bc = BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(&title, Style::default().fg(Color::Cyan))))
        .data(&data)
        .bar_width(3)
        .bar_gap(1)
        .value_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
        )
        .style(Style::default().fg(Color::Green));
    f.render_widget(count_bc, area);
}
