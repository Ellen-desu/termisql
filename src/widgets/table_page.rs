use crate::widgets::Component;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Padding, Paragraph, Widget},
};

#[derive(Default)]
pub struct TablePage {
    pub page: u16,
    pub end: u16,
    pub size: u8,
}

impl TablePage {
    pub fn with_size(mut self, size: u8) -> Self {
        self.size = size;
        self
    }
    pub fn reset(&mut self) {
        self.page = 0;
        self.end = 0;
    }
}

impl Component for TablePage {
    fn render(&mut self, area: Rect, buf: &mut Buffer, focus: bool) {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1))
            .border_style(if focus {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
            .render(area, buf);

        let [_, area, _] = Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .areas(area);

        Paragraph::new(format!("↑ {}/{} ↓", self.page, self.end))
            .centered()
            .render(area, buf);
    }

    fn next(&mut self) {
        if self.page < self.end {
            self.page += 1;
        }
    }

    fn prev(&mut self) {
        if self.page > 1 {
            self.page -= 1;
        }
    }
}
