use crate::widgets::Component;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, BorderType, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget,
    },
};

#[derive(Default)]
pub struct TableView {
    pub items: Option<(Vec<String>, Vec<Vec<String>>)>,
    pub state: TableState,
}

impl Component for TableView {
    fn render(&mut self, area: Rect, buf: &mut Buffer, focus: bool) {
        let block = Block::bordered()
            .title(" View ")
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1))
            .border_style(if focus {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        match &self.items {
            Some(items) => {
                let rows = items
                    .1
                    .iter()
                    .map(|items| {
                        Row::new(
                            items
                                .iter()
                                .map(|item| item.as_str())
                                .collect::<Vec<&str>>(),
                        )
                    })
                    .collect::<Vec<Row>>();

                StatefulWidget::render(
                    Table::new(
                        rows,
                        (0..items.0.len())
                            .map(|_| Constraint::Fill(1))
                            .collect::<Vec<Constraint>>(),
                    )
                    .block(block)
                    .header(
                        Row::new(
                            items
                                .0
                                .iter()
                                .map(|key| key.as_str())
                                .collect::<Vec<&str>>(),
                        )
                        .style(Modifier::BOLD)
                        .bottom_margin(1),
                    )
                    .highlight_symbol(">> ")
                    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD)),
                    area,
                    buf,
                    &mut self.state,
                );
            }
            None => {
                Widget::render(
                    Paragraph::new("Please select a table.")
                        .block(block)
                        .centered(),
                    area,
                    buf,
                );
            }
        };
    }

    fn next(&mut self) {
        if let Some((_, rows)) = &self.items
            && let Some(n) = self.state.selected()
            && !rows.is_empty()
        {
            self.state.select(Some(n + 1));
        } else {
            self.state.select(Some(0));
        }
    }

    fn prev(&mut self) {
        if let Some((_, rows)) = &self.items
            && let Some(n) = self.state.selected()
            && !rows.is_empty()
            && n > 0
        {
            self.state.select(Some(n - 1));
        } else {
            self.state.select(Some(0));
        }
    }
}
