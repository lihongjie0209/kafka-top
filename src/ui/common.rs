use crate::state::LoadState;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

/// Render a loading spinner centered in the given area.
pub fn render_loading(frame: &mut Frame, area: Rect, message: &str) {
    let text = Text::from(Line::from(Span::styled(
        format!(" {} ...", message),
        Style::default().fg(Color::Yellow),
    )));
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::bordered());
    frame.render_widget(paragraph, area);
}

/// Render a red error box.
pub fn render_error(frame: &mut Frame, area: Rect, error: &str) {
    let block = Block::bordered()
        .border_style(Style::default().fg(Color::Red))
        .title(" Error ")
        .title_alignment(Alignment::Center);

    let text = Text::from(Line::from(Span::styled(
        error,
        Style::default().fg(Color::Red),
    )));
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

/// Render an empty state message.
pub fn render_empty(frame: &mut Frame, area: Rect, message: &str) {
    let text = Text::from(Line::from(Span::styled(
        format!(" {} ", message),
        Style::default().fg(Color::DarkGray),
    )));
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::bordered());
    frame.render_widget(paragraph, area);
}

/// Delegate rendering based on LoadState
pub fn render_load_state<T, F>(frame: &mut Frame, area: Rect, state: &LoadState<T>, loading_msg: &str, render_fn: F)
where
    T: Clone,
    F: FnOnce(&mut Frame, Rect, &T),
{
    match state {
        LoadState::Idle => render_empty(frame, area, "Press [r] to refresh"),
        LoadState::Loading => render_loading(frame, area, loading_msg),
        LoadState::Error(e) => render_error(frame, area, e),
        LoadState::Loaded(data) => render_fn(frame, area, data),
    }
}

/// Format a large number with commas (e.g. 12345 -> "12,345")
pub fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
