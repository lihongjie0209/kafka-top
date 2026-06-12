use crate::app::App;
use crate::state::GroupsData;
use crate::ui::common::render_load_state;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    render_load_state(frame, area, &app.groups_state, "Fetching consumer groups", |f, area, data| {
        render_groups_table(f, area, data, app);
    });
}

fn render_groups_table(frame: &mut Frame, area: Rect, data: &GroupsData, app: &App) {
    let title = format!(" Consumer Groups ({} total)", data.groups.len());

    let header_cells = ["Group ID", "State", "Protocol Type", "Members"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells);

    let rows = data.groups.iter().enumerate().map(|(i, group)| {
        let style = if i == app.group_list_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Color the state
        let state_style = match group.state.as_str() {
            "Stable" => Style::default().fg(Color::Green),
            "Empty" => Style::default().fg(Color::DarkGray),
            _ => Style::default().fg(Color::Yellow),
        };

        let cells = vec![
            Cell::from(group.group_id.clone()),
            Cell::from(group.state.clone()).style(state_style),
            Cell::from(group.protocol_type.clone()),
            Cell::from(group.member_count.to_string()),
        ];
        Row::new(cells).style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Min(25),
            Constraint::Length(12),
            Constraint::Length(15),
            Constraint::Length(10),
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
