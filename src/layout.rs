use color_eyre::Result;
use ratatui::layout::{Constraint, Layout, Rect};

pub struct UILayout {
    pub list_area: Rect,
    pub table_area: Rect,
    pub page_area: Rect,
}

impl UILayout {
    pub fn new(area: Rect) -> Result<Self> {
        let [list_area, _, raw_table_area] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Length(1),
            Constraint::Percentage(75),
        ])
        .areas(area);

        let [table_area, _, page_area] = Layout::vertical([
            Constraint::Percentage(90),
            Constraint::Length(1),
            Constraint::Percentage(10),
        ])
        .areas(raw_table_area);

        Ok(Self {
            list_area,
            table_area,
            page_area,
        })
    }
}
