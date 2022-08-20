use super::app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{self, Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, LineGauge, List, ListItem, Paragraph, Row,
        Sparkline, Table, Tabs, Wrap,
    },
    Frame, Terminal,
};

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());
    let titles = app
        .tabs
        .titles
        .iter()
        .map(|t| Spans::from(Span::styled(t, Style::default().fg(Color::Green))))
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(Span::styled(
            app.title.to_string(),
            Style::default().fg(Color::Yellow),
        )))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    f.render_widget(tabs, chunks[0]);
    match app.tabs.index {
        0 => draw_first_tab(f, app, chunks[1]),
        1 => draw_second_tab(f, app, chunks[1]),
        _ => (),
    };
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(9),
                Constraint::Min(8),
                Constraint::Length(7),
            ]
            .as_ref(),
        )
        .split(area);

    draw_gauges(f, app, chunks[0]);
    draw_lists(f, app, chunks[1]);
    draw_text(f, chunks[2]);
}

fn draw_second_tab<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
    let text = vec![
        Spans::from(""),
        Spans::from("My social accounts:"),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("Instagram: ", Style::default().fg(Color::Yellow)),
            Span::styled("@0xhades", Style::default().fg(Color::Cyan)),
        ]),
        Spans::from(vec![
            Span::styled("Twitter: ", Style::default().fg(Color::Yellow)),
            Span::styled("@0x0hades", Style::default().fg(Color::Cyan)),
        ]),
        Spans::from(vec![
            Span::styled("Github: ", Style::default().fg(Color::Yellow)),
            Span::styled("@0xhades", Style::default().fg(Color::Cyan)),
        ]),
    ];
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "About coder",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_lists<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .direction(Direction::Horizontal)
        .split(area);

    // Draw logs
    let info_style = Style::default().fg(Color::Blue);
    let success_style = Style::default().fg(Color::Green);
    let warning_style = Style::default().fg(Color::Yellow);
    let error_style = Style::default().fg(Color::Magenta);
    let critical_style = Style::default().fg(Color::Red);
    let logs: Vec<ListItem> = app
        .logs
        .items
        .iter()
        .map(|(level, evt)| {
            let s = match level.as_str() {
                "SUCCESS" => success_style,
                "ERROR" => error_style,
                "CRITICAL" => critical_style,
                "WARNING" => warning_style,
                _ => info_style,
            };
            let content = vec![Spans::from(vec![
                //Span::styled(format!("{:<9}", level), s),
                //Span::raw(evt),
                Span::styled(evt, s),
            ])];
            ListItem::new(content)
        })
        .collect();
    let logs = List::new(logs).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Logs", Style::default().fg(Color::Yellow))),
    );
    f.render_stateful_widget(logs, chunks[0], &mut app.logs.state);

    // Draw hunt
    let hunt: Vec<ListItem> = app
        .hunt
        .items
        .iter()
        .map(|i| {
            ListItem::new(vec![Spans::from(Span::styled(
                i,
                Style::default().fg(Color::LightGreen),
            ))])
        })
        .collect();
    let hunt = List::new(hunt).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Hunts", Style::default().fg(Color::Green))),
    );
    f.render_stateful_widget(hunt, chunks[1], &mut app.hunt.state);

    // Draw taken
    let taken: Vec<ListItem> = app
        .takens
        .items
        .iter()
        .map(|i| {
            ListItem::new(vec![Spans::from(Span::styled(
                i,
                Style::default().fg(Color::LightMagenta),
            ))])
        })
        .collect();
    let taken = List::new(taken).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Taken", Style::default().fg(Color::Magenta))),
    );
    f.render_stateful_widget(taken, chunks[2], &mut app.takens.state);

    // Draw error
    let error: Vec<ListItem> = app
        .errors
        .items
        .iter()
        .map(|i| {
            ListItem::new(vec![Spans::from(Span::styled(
                i,
                Style::default().fg(Color::LightRed),
            ))])
        })
        .collect();
    let error = List::new(error).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Error/Miss", Style::default().fg(Color::Red))),
    );
    f.render_stateful_widget(error, chunks[3], &mut app.errors.state);
}

fn draw_gauges<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Length(2), Constraint::Length(3)].as_ref())
        .margin(1)
        .split(area);
    let block = Block::default().borders(Borders::ALL).title("Status");
    f.render_widget(block, area);

    let label = format!("{:.2}%", app.progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().title("Used Sessions:"))
        .gauge_style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Black)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .label(label)
        .ratio(app.progress);
    f.render_widget(gauge, chunks[0]);

    let text = vec![
        Spans::from(vec![
            Span::styled("Hunts: ", Style::default().fg(Color::LightGreen)),
            Span::raw(app.hunt.items.len().to_string()),
        ]),
        Spans::from(vec![
            Span::styled("Taken: ", Style::default().fg(Color::LightMagenta)),
            Span::raw(app.taken.to_string()),
        ]),
        Spans::from(vec![
            Span::styled("Errors: ", Style::default().fg(Color::LightRed)),
            Span::raw(app.error.to_string()),
        ]),
        Spans::from(vec![
            Span::styled("Total Attempts: ", Style::default().fg(Color::Yellow)),
            Span::raw((app.taken + app.hunt.items.len() + app.miss).to_string()),
        ]),
        Spans::from(vec![
            Span::styled(
                "Requests Per Seconds: ",
                Style::default().fg(Color::LightBlue),
            ),
            Span::raw(app.requests_per_seconds.to_string()),
        ]),
    ];

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[1]);
}

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let text = vec![
        Spans::from("Keyboard Usage:"),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("q: ", Style::default().fg(Color::Red)),
            Span::raw("Quits the application"),
        ]),
        Spans::from(vec![
            Span::styled("right arrow (", Style::default().fg(Color::Cyan)),
            Span::styled("->", Style::default().fg(Color::Yellow)),
            Span::styled("): ", Style::default().fg(Color::Cyan)),
            Span::raw("Go to the next tab"),
        ]),
        Spans::from(vec![
            Span::styled("left arrow (", Style::default().fg(Color::Cyan)),
            Span::styled("<-", Style::default().fg(Color::Yellow)),
            Span::styled("): ", Style::default().fg(Color::Cyan)),
            Span::raw("back to the previous tab"),
        ]),
    ];
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Keymap",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
