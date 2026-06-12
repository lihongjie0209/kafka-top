mod common;
mod dashboard;
mod group_detail;
mod groups;
mod layout;
mod tabs;
mod topic_detail;
mod topics;

use crate::app::{App, Tab, View};
use ratatui::prelude::Stylize;
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let layout = layout::create_layout(frame.area());

    // Title bar
    let title = format!(" kafka-top  |  {} ", app.cli.bootstrap_servers);
    frame.render_widget(
        ratatui::widgets::Paragraph::new(title).style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        layout.title_bar,
    );

    // Content area
    match app.current_view {
        View::Normal => match app.current_tab {
            Tab::Dashboard => dashboard::render(frame, layout.content, app),
            Tab::Topics => topics::render(frame, layout.content, app),
            Tab::ConsumerGroups => groups::render(frame, layout.content, app),
        },
        View::TopicDetail => topic_detail::render(frame, layout.content, app),
        View::GroupDetail => group_detail::render(frame, layout.content, app),
    }

    // Tab bar
    tabs::render(frame, layout.tab_bar, app.current_tab, app.current_view);

    // Status line
    let help = match app.current_view {
        View::TopicDetail | View::GroupDetail => " [Esc] Back  [r] Refresh  [q] Quit ",
        _ => " [Tab/Shift+Tab] Navigate  [Enter] Details  [r] Refresh  [q] Quit ",
    };
    frame.render_widget(
        ratatui::widgets::Paragraph::new(ratatui::text::Span::styled(
            help,
            ratatui::style::Style::default().dim(),
        )),
        layout.status_line,
    );
}
