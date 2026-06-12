use crate::app::{Tab, View};
use ratatui::layout::Rect;
use ratatui::prelude::Stylize;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, selected_tab: Tab, current_view: View) {
    let titles = [" Dashboard ", " Topics ", " Consumer Groups "];
    let labels: Vec<Line> = titles
        .iter()
        .map(|t| {
            let s = Span::styled(*t, Style::default().fg(ratatui::style::Color::White));
            Line::from(s)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().dim());

    // Highlight style for selected tab
    let highlight_style = if current_view != View::Normal {
        // dimmed when in detail view
        Style::default().fg(ratatui::style::Color::DarkGray)
    } else {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(ratatui::style::Color::Yellow)
    };

    let tabs = Tabs::new(labels)
        .select(selected_tab as usize)
        .highlight_style(highlight_style)
        .divider("│")
        .block(block);

    frame.render_widget(tabs, area);
}
