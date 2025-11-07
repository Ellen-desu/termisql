mod table_list;
mod table_page;
mod table_view;

use ratatui::{buffer::Buffer, layout::Rect};
pub use table_list::TableList;
pub use table_page::TablePage;
pub use table_view::TableView;

pub trait Component {
    fn render(&mut self, area: Rect, buf: &mut Buffer, focus: bool);

    fn next(&mut self);

    fn prev(&mut self);
}
