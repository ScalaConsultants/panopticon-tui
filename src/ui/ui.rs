use std::io;

use tui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    Terminal,
    widgets::{Axis, BarChart, Block, Borders, Chart, Dataset, Paragraph, List, Tabs, Text},
};

use crate::App;
use crate::ui::app::{SlickTab, TabKind, ZMXTab, AkkaActorTreeTab};
use crate::jmx_client::model::HikariMetrics;
use crate::zio::model::FiberCount;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), io::Error> {
    terminal.draw(|mut f| {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        let tabs = app.tabs.to_owned();
        let titles = tabs.titles();
        let tabs_widget = Tabs::default()
            .block(Block::default()
                .borders(Borders::ALL)
                .title_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD))
                .title(app.title))
            .titles(&titles)
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(tabs.index);
        f.render_widget(tabs_widget, chunks[0]);
        match tabs.current().kind {
            TabKind::ZMX => &app.zmx.as_mut().map(|mut t| draw_zio_tab(&mut f, &mut t, chunks[1])),
            TabKind::Slick => &app.slick.as_ref().map(|t| draw_slick_tab(&mut f, t, chunks[1])),
            TabKind::AkkaActorTree => &app.actor_tree.as_mut().map(|t| draw_actor_tree_tab(&mut f, t, chunks[1])),
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
    let p = Paragraph::new(text.iter())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("by Scalac Sp.z o.o.")
                .title_style(Style::default().fg(Color::Magenta).modifier(Modifier::BOLD)),
        )
        .wrap(true);
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
            .title_style(Style::default().fg(Color::Cyan))
            .title(&active_threads_title))
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
            .title_style(Style::default().fg(Color::Cyan))
            .title(&queue_size_title))
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

    let title = format!(
        "HikariCP (total={}, active={}, idle={}, waiting={})",
        total_connections,
        active_connections,
        idle_connections,
        waiting_connections
    );
    let label = &["0".to_owned(), ((max_connections as f64) / 2.0).to_string(), max_connections.to_string()];
    let c = Chart::default()
        .block(
            Block::default()
                .title(&title)
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
                .labels(label)
        )
        .datasets(&datasets);
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

                let items = zmx.fibers.items.iter().map(|i| Text::raw(i));

                let list = List::new(items)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default().fg(Color::Cyan))
                        .title("Fibers (press <Enter> to take a snapshot)"))
                    .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
                    .highlight_symbol(">");
                f.render_stateful_widget(list, chunks[0], &mut zmx.fibers.state);

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

                let title = format!(
                    "Fibers (total={}, running={}, done={}, finishing={}, suspended={})",
                    total_fibers,
                    running_fibers,
                    done_fibers,
                    finishing_fibers,
                    suspended_fibers
                );
                let label = &["0".to_owned(), ((max_fibers as f64) / 2.0).to_string(), max_fibers.to_string()];
                let c = Chart::default()
                    .block(
                        Block::default()
                            .title(&title)
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
                            .labels(label)
                    )
                    .datasets(&datasets);
                f.render_widget(c, chunks[1]);
            }

            let text = [Text::raw(zmx.selected_fiber_dump.0.to_owned())];

            let p = Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Fiber dump (press <PageUp>/<PageDown> to scroll)")
                        .title_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(true)
                .scroll(zmx.scroll);
            f.render_widget(p, chunks[1]);
        }
    }
}

fn draw_actor_tree_tab<B>(f: &mut Frame<B>, tab: &mut AkkaActorTreeTab, area: Rect)
    where B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Min(7), Constraint::Length(3)].as_ref())
        .split(area);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(chunks[0]);
        draw_actor_tree(f, tab, chunks[0]);
        draw_actor_count_chart(f, tab, chunks[1]);
    }
    draw_text(f, chunks[1]);
}


fn draw_actor_tree<B>(f: &mut Frame<B>, tab: &mut AkkaActorTreeTab, area: Rect)
    where B: Backend,
{
    let items = tab.actors.items.iter().map(|i| Text::raw(i));

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title_style(Style::default().fg(Color::Cyan))
            .title("Actors (press <Enter> to reload the tree)"))
        .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
        .highlight_symbol(">");

    f.render_stateful_widget(list, area, &mut tab.actors.state);
}

fn draw_actor_count_chart<B>(f: &mut Frame<B>, tab: &AkkaActorTreeTab, area: Rect)
    where B: Backend,
{
    let data: Vec<(&str, u64)> = tab.actor_counts.iter()
        .map(|x| ("", x.to_owned()))
        .collect();

    let title = format!("Running actors: {}", tab.actor_counts.back().unwrap_or(&0));
    let count_bc = BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title_style(Style::default().fg(Color::Cyan))
            .title(&title))
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
