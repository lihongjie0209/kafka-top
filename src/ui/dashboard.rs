use crate::app::App;
use crate::state::DashboardData;
use crate::ui::common::render_load_state;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    render_load_state(frame, area, &app.dashboard_state, "Fetching cluster info", |f, area, data| {
        render_dashboard(f, area, data);
    });
}

fn render_dashboard(frame: &mut Frame, area: Rect, data: &DashboardData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    // Metrics section
    render_metrics(frame, chunks[0], data);

    // Brokers table
    render_brokers(frame, chunks[1], data);
}

fn render_metrics(frame: &mut Frame, area: Rect, data: &DashboardData) {
    let text = vec![
        Line::from(vec![
            Span::styled("Connected to: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&data.connected_broker),
        ]),
        Line::from(vec![
            Span::styled("Brokers: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(data.broker_count.to_string()),
            Span::raw("  |  "),
            Span::styled("Topics: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(data.topic_count.to_string()),
            Span::raw("  |  "),
            Span::styled("Consumer Groups: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(data.group_count.to_string()),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Cluster Metrics ")
            .border_style(Style::default()),
    );
    frame.render_widget(paragraph, area);
}

fn render_brokers(frame: &mut Frame, area: Rect, data: &DashboardData) {
    let header_cells = ["ID", "Host", "Port"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells);

    let rows = data.brokers.iter().map(|broker| {
        let cells = vec![
            Cell::from(broker.id.to_string()),
            Cell::from(broker.host.clone()),
            Cell::from(broker.port.to_string()),
        ];
        Row::new(cells)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Brokers "),
    );

    frame.render_widget(table, area);
}
