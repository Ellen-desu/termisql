use crate::widgets::Component;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, List, ListState, Padding, Paragraph, StatefulWidget, Widget},
};

#[derive(Default)]
pub struct TableList {
    pub items: Vec<String>,
    pub state: ListState,
}

impl Component for TableList {
    fn render(&mut self, area: Rect, buf: &mut Buffer, focus: bool) {
        let block = Block::bordered()
            .title(" Tables ")
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1))
            .border_style(if focus {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        if self.items.is_empty() {
            Widget::render(
                Paragraph::new("No such table.").centered().block(block),
                area,
                buf,
            );
        } else {
            StatefulWidget::render(
                List::new(
                    self.items
                        .iter()
                        .map(|item| item.as_str())
                        .collect::<Vec<&str>>(),
                )
                .block(block)
                .highlight_symbol(">> ")
                .highlight_style(Style::default().add_modifier(Modifier::BOLD)),
                area,
                buf,
                &mut self.state,
            );
        }
    }

    fn next(&mut self) {
        if let Some(n) = self.state.selected()
            && (n + 1) < self.items.len()
        {
            self.state.select(Some(n + 1));
        } else {
            self.state.select(Some(0));
        }
    }

    fn prev(&mut self) {
        if let Some(n) = self.state.selected()
            && n > 0
        {
            self.state.select(Some(n - 1));
        } else {
            self.state.select(Some(self.items.len() - 1));
        }
    }
}
