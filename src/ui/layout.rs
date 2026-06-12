use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct ScreenLayout {
    pub title_bar: Rect,
    pub content: Rect,
    pub tab_bar: Rect,
    pub status_line: Rect,
}

pub fn create_layout(area: Rect) -> ScreenLayout {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),     // title
            Constraint::Min(0),        // content
            Constraint::Length(3),     // tab bar
            Constraint::Length(1),     // status
        ])
        .split(area);

    ScreenLayout {
        title_bar: chunks[0],
        content: chunks[1],
        tab_bar: chunks[2],
        status_line: chunks[3],
    }
}
