use crate::app::App;
use crate::state::TopicsData;
use crate::ui::common::{format_number, render_load_state};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    render_load_state(frame, area, &app.topics_state, "Fetching topics", |f, area, data| {
        render_topics_table(f, area, data, app);
    });
}

fn render_topics_table(frame: &mut Frame, area: Rect, data: &TopicsData, app: &App) {
    let title = format!(" Topics ({} total)", data.topics.len());

    let header_cells = ["Name", "Partitions", "Total Messages"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells);

    let rows = data.topics.iter().enumerate().map(|(i, topic)| {
        let style = if i == app.topic_list_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let cells = vec![
            Cell::from(topic.name.clone()),
            Cell::from(topic.partition_count.to_string()),
            Cell::from(format_number(topic.total_messages)),
        ];
        Row::new(cells).style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Min(25),
            Constraint::Length(12),
            Constraint::Length(18),
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
