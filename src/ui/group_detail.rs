use crate::app::App;
use crate::state::GroupDetailData;
use crate::ui::common::render_load_state;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    render_load_state(frame, area, &app.group_detail_state, "Fetching group detail", |f, area, data| {
        render_group_detail(f, area, data);
    });
}

fn render_group_detail(frame: &mut Frame, area: Rect, data: &GroupDetailData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // group summary
            Constraint::Length(min_members_section(data.members.len())),
            Constraint::Min(0),    // assignments table
        ])
        .split(area);

    render_group_summary(frame, chunks[0], data);
    render_members(frame, chunks[1], data);
    render_assignments(frame, chunks[2], data);
}

fn min_members_section(member_count: usize) -> u16 {
    // header (1) + each member row (1) + border (2) = 3 + member_count
    3u16 + member_count.min(5) as u16
}

fn render_group_summary(frame: &mut Frame, area: Rect, data: &GroupDetailData) {
    let state_style = match data.state.as_str() {
        "Stable" => Style::default().fg(Color::Green),
        "Empty" => Style::default().fg(Color::DarkGray),
        _ => Style::default().fg(Color::Yellow),
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Group ID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&data.group_id),
        ]),
        Line::from(vec![
            Span::styled("State: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(&data.state, state_style),
            Span::raw("  |  "),
            Span::styled("Protocol: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&data.protocol),
            Span::raw("  |  "),
            Span::styled("Members: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(data.members.len().to_string()),
            Span::raw("  |  "),
            Span::styled("Partitions: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(data.assignments.len().to_string()),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Group Summary "),
    );
    frame.render_widget(paragraph, area);
}

fn render_members(frame: &mut Frame, area: Rect, data: &GroupDetailData) {
    let header_cells = ["Member ID", "Client ID", "Host"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells);

    let rows = data.members.iter().map(|m| {
        Row::new(vec![
            Cell::from(m.member_id.clone()),
            Cell::from(m.client_id.clone()),
            Cell::from(m.client_host.clone()),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Min(15),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Members "));

    frame.render_widget(table, area);
}

fn render_assignments(frame: &mut Frame, area: Rect, data: &GroupDetailData) {
    let header_cells = ["Topic", "Partition", "Current Offset", "Log End Offset", "Lag", "Rate (msg/s)"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells);

    let rows = data.assignments.iter().map(|a| {
        let lag_style = if a.lag > 0 {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };

        let rate_str = if a.consumer_rate > 0.0 {
            format!("{:.1}", a.consumer_rate)
        } else {
            "-".to_string()
        };

        Row::new(vec![
            Cell::from(a.topic.clone()),
            Cell::from(a.partition_id.to_string()),
            Cell::from(a.current_offset.to_string()),
            Cell::from(a.log_end_offset.to_string()),
            Cell::from(a.lag.to_string()).style(lag_style),
            Cell::from(rate_str),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(16),
            Constraint::Length(18),
            Constraint::Length(10),
            Constraint::Length(16),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Partition Assignments & Lag "),
    );

    frame.render_widget(table, area);
}
