
use std::io;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
//use tui::widgets::{Block, Borders, List, Paragraph, SelectableList, Tabs, Text, Widget};

use tui::{
    backend::Backend, 
    Frame,
    Terminal,
};

use tui::widgets::{
    BarChart, Block, Borders, List, Paragraph, SelectableList, Tabs, Text, Widget,
};


use crate::App;

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
            .titles(&app.tabs.titles)
            .style(Style::default().fg(Color::Green))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(app.tabs.index)
            .render(&mut f, chunks[0]);
        match app.tabs.index {
            0 => draw_first_tab(&mut f, &app, chunks[1]),
            2 => draw_zio_tab(&mut f, &app, chunks[1]),
            _ => {}
        };
    })
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Min(7),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);
//    draw_gauges(f, app, chunks[0]);
    draw_zk_nodes_list(f, app, chunks[0]);
    draw_text(f, chunks[1]);
}

fn draw_zk_nodes_list<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
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
            SelectableList::default()
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                    .title("Ensemble"))
                .items(&app.zookeeper_nodes.items)
                .select(Some(app.zookeeper_nodes.selected))
                .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
                .highlight_symbol(">")
                .render(f, chunks[0]);

            let items = app.zookeeper_wchc.iter().map(|&it| {
                Text::styled(format!("{}", it), Style::default().fg(Color::White))
            });

            List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                    .title("WCHC"))
                .render(f, chunks[1]);
        }
    }
}

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let text = [
        Text::raw("Contact us: "),
        Text::styled("panopticon@scalac.io", Style::default().fg(Color::Blue)),
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

fn draw_zio_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Min(7),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);
//    draw_gauges(f, app, chunks[0]);
    draw_fiber_list(f, app, chunks[0]);
    draw_text(f, chunks[1]);
}

fn draw_fiber_list<B>(f: &mut Frame<B>, app: &App, area: Rect)
where
    B: Backend,
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
                    .items(&app.fibers.items)
                    .select(Some(app.fibers.selected))
                    .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
                    .highlight_symbol(">")
                    .render(f, chunks[0]);

                BarChart::default()
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title_style(Style::default().fg(Color::Cyan))
                        .title("Fiber count"))
                    .data(&app.barchart)
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
            let text = [Text::raw(app.fiber_dump)];

            Paragraph::new(text.iter())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Fiber dump")
                        .title_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(true)
                .scroll(app.scroll)
                .render(f, chunks[1]);
        }
    }
}
