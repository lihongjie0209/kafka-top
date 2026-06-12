use crate::app::App;
use crate::state::TopicDetailData;
use crate::ui::common::{format_number, render_load_state};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    render_load_state(frame, area, &app.topic_detail_state, "Fetching topic detail", |f, area, data| {
        render_topic_detail(f, area, data);
    });
}

fn render_topic_detail(frame: &mut Frame, area: Rect, data: &TopicDetailData) {
    let title = format!(" Topic: {} ({} partitions)", data.topic, data.partitions.len());

    let header_cells = ["Partition", "Leader", "Replicas", "ISR", "Log End Offset", "Rate (msg/s)"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells);

    let rows = data.partitions.iter().map(|p| {
        let replicas: Vec<String> = p.replicas.iter().map(|r| r.to_string()).collect();
        let isr: Vec<String> = p.isr.iter().map(|i| i.to_string()).collect();

        let rate_str = if p.producer_rate > 0.0 {
            format!("{:.1}", p.producer_rate)
        } else {
            "-".to_string()
        };

        let cells = vec![
            Cell::from(p.id.to_string()).style(
                if p.leader == p.id as i32 || p.replicas.len() <= 1 {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                }
            ),
            Cell::from(p.leader.to_string()).style(
                if p.leader >= 0 {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                }
            ),
            Cell::from(replicas.join(", ")),
            Cell::from(isr.join(", ")),
            Cell::from(format_number(p.log_end_offset)),
            Cell::from(rate_str),
        ];
        Row::new(cells)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Min(12),
            Constraint::Min(12),
            Constraint::Length(18),
            Constraint::Length(16),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title),
    );

    frame.render_widget(table, area);
}
