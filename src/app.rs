use crate::{
    args::Args,
    db::Database::{self, MySQL, SQLite},
    layout::UILayout,
    widgets::{Component, TableList, TablePage, TableView},
};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use sqlx::{QueryBuilder, Row, TypeInfo, Value, ValueRef, mysql::MySqlRow, sqlite::SqliteRow};

use std::time::Duration;
use tokio::sync::{mpsc, watch};

#[derive(PartialEq, Debug)]
enum CurrentScreen {
    Main,
    Selecting,
    Viewing,
    Paging,
}

pub struct App {
    table_list: TableList,
    table_view: TableView,
    table_page: TablePage,
    request_redraw: bool,
    request_update_data: bool,
    screen: CurrentScreen,
    exit: bool,
    db: Database,
}

impl App {
    pub async fn build(args: Args) -> Result<Self> {
        Ok(Self {
            table_list: TableList::default(),
            table_view: TableView::default(),
            table_page: TablePage::default().with_size(args.page_size),
            request_redraw: true,
            request_update_data: true,
            screen: CurrentScreen::Main,
            exit: false,
            db: Database::connect(&args).await?,
        })
    }

    fn draw(&mut self, frame: &mut Frame) {
        let layout = UILayout::new(frame.area()).unwrap();
        let buf = frame.buffer_mut();

        self.table_list.render(
            layout.list_area,
            buf,
            self.screen == CurrentScreen::Selecting,
        );

        self.table_view.render(
            layout.table_area,
            buf,
            self.screen == CurrentScreen::Viewing,
        );

        self.table_page
            .render(layout.page_area, buf, self.screen == CurrentScreen::Paging);
    }

    fn clamp_selection(&mut self, selected: &mut usize) {
        let len = self.table_list.items.len();
        if len == 0 {
            self.table_list.state.select(None);
        } else if *selected >= len {
            *selected = len - 1;
            self.table_list.state.select(Some(*selected));
        }
    }

    fn restore_widgets(&mut self) {
        self.table_page.reset();
        self.table_view = TableView::default();
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit = true,
            KeyCode::Enter if self.screen == CurrentScreen::Main => {
                self.screen = CurrentScreen::Selecting
            }
            KeyCode::Right => match self.screen {
                CurrentScreen::Selecting => self.screen = CurrentScreen::Viewing,
                CurrentScreen::Viewing => self.screen = CurrentScreen::Paging,
                CurrentScreen::Paging => self.screen = CurrentScreen::Selecting,
                _ => (),
            },
            KeyCode::Left => match self.screen {
                CurrentScreen::Selecting => self.screen = CurrentScreen::Paging,
                CurrentScreen::Paging => self.screen = CurrentScreen::Viewing,
                CurrentScreen::Viewing => self.screen = CurrentScreen::Selecting,
                _ => (),
            },
            KeyCode::Esc if self.screen != CurrentScreen::Main => self.screen = CurrentScreen::Main,
            KeyCode::Up => {
                match self.screen {
                    CurrentScreen::Selecting if !self.table_list.items.is_empty() => {
                        self.table_list.prev();
                        self.request_update_data = true
                    }
                    CurrentScreen::Viewing => self.table_view.prev(),
                    CurrentScreen::Paging
                        if self.table_page.page != 0 && self.table_page.end != 0 =>
                    {
                        self.table_page.prev();
                        self.request_update_data = true;
                    }
                    _ => (),
                };
            }
            KeyCode::Down => {
                match self.screen {
                    CurrentScreen::Selecting if !self.table_list.items.is_empty() => {
                        self.table_list.next();
                        self.request_update_data = true;
                    }
                    CurrentScreen::Viewing => self.table_view.next(),
                    CurrentScreen::Paging
                        if self.table_page.page != 0 && self.table_page.end != 0 =>
                    {
                        self.table_page.next();
                        self.request_update_data = true;
                    }
                    _ => (),
                };
            }
            _ => (),
        };

        self.request_redraw = true;
    }

    async fn update_data(&mut self) -> Result<()> {
        match &self.db {
            SQLite(pool) => {
                let mut tx = pool.begin().await?;

                self.table_list.items = sqlx::query("SELECT name FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
                	.map(|row: SqliteRow| row.get(0))
                	.fetch_all(&mut *tx)
                	.await?;

                match self.table_list.state.selected() {
                    Some(mut selected) => {
                        self.clamp_selection(&mut selected);

                        if self.table_list.state.selected().is_none() {
                            self.restore_widgets();
                            return Ok(());
                        }

                        let tablename = &self.table_list.items[selected];

                        let rows_count: i64 =
                            sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", tablename))
                                .fetch_one(&mut *tx)
                                .await?;

                        self.table_page.end = ((rows_count as f64 / self.table_page.size as f64)
                            .ceil() as u16)
                            .max(1);
                        self.table_page.page = if self.table_page.page > self.table_page.end {
                            self.table_page.end
                        } else {
                            self.table_page.page.max(1)
                        };

                        let columns = sqlx::query(&format!(
                            "SELECT name FROM PRAGMA_TABLE_INFO('{}')",
                            tablename
                        ))
                        .map(|row: SqliteRow| row.get::<String, _>(0))
                        .fetch_all(&mut *tx)
                        .await?;

                        let mut qb = QueryBuilder::new(&format!(
                            "SELECT {} FROM {} LIMIT ",
                            columns.join::<&str>(", "),
                            tablename
                        ));

                        qb.push_bind(self.table_page.size)
                            .push(" OFFSET ")
                            .push_bind((self.table_page.page - 1) * self.table_page.size as u16);

                        let rows: Vec<Vec<String>> = qb
                            .build()
                            .map(|row: SqliteRow| {
                                let mut records: Vec<String> = Vec::new();

                                for idx in 0..row.len() {
                                    let value = row.try_get_raw(idx).unwrap().to_owned();

                                    let value_str = if value.is_null() {
                                        "NULL".into()
                                    } else {
                                        match value.type_info().name() {
                                            "INTEGER" => value
                                                .try_decode::<i64>()
                                                .map_or("<err>".into(), |v| v.to_string()),
                                            "REAL" => value
                                                .try_decode::<f64>()
                                                .map_or("<err>".into(), |v| v.to_string()),
                                            "TEXT" => value
                                                .try_decode::<String>()
                                                .unwrap_or("<err>".into()),
                                            _ => "<unsupported>".into(),
                                        }
                                    };

                                    records.push(value_str);
                                }

                                records
                            })
                            .fetch_all(&mut *tx)
                            .await?;

                        self.table_view.items = Some((columns, rows));
                    }
                    None => self.restore_widgets(),
                };
            }
            MySQL(pool) => {
                let mut tx = pool.begin().await?;

                self.table_list.items = sqlx::query("SELECT table_name FROM INFORMATION_SCHEMA.TABLES WHERE table_schema = DATABASE()")
                	.map(|row: MySqlRow| row.get(0))
                   	.fetch_all(&mut *tx)
                    .await?;

                match self.table_list.state.selected() {
                    Some(mut selected) => {
                        self.clamp_selection(&mut selected);

                        if self.table_list.state.selected().is_none() {
                            self.restore_widgets();
                            return Ok(());
                        }

                        let tablename = &self.table_list.items[selected];

                        let rows_count: i64 =
                            sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", tablename))
                                .fetch_one(&mut *tx)
                                .await?;

                        self.table_page.end = ((rows_count as f64 / self.table_page.size as f64)
                            .ceil() as u16)
                            .max(1);
                        self.table_page.page = if self.table_page.page > self.table_page.end {
                            self.table_page.end
                        } else {
                            self.table_page.page.max(1)
                        };

                        let mut qb = QueryBuilder::new(
                            "SELECT COLUMN_NAME
                        FROM INFORMATION_SCHEMA.COLUMNS
                        WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ",
                        );
                        qb.push_bind(tablename);

                        let columns = qb
                            .build()
                            .map(|row: MySqlRow| row.get::<String, _>(0))
                            .fetch_all(&mut *tx)
                            .await?;

                        let mut qb = QueryBuilder::new(&format!(
                            "SELECT {} FROM {} LIMIT ",
                            columns.join::<&str>(", "),
                            tablename
                        ));

                        qb.push_bind(self.table_page.size)
                            .push(" OFFSET ")
                            .push_bind((self.table_page.page - 1) * self.table_page.size as u16);

                        let rows: Vec<Vec<String>> = qb
                            .build()
                            .map(|row: MySqlRow| {
                                let mut records = Vec::new();

                                for idx in 0..row.len() {
                                    let value = ValueRef::to_owned(&row.try_get_raw(idx).unwrap());

                                    let value_str = if value.is_null() {
                                        "NULL".into()
                                    } else {
                                        match value.type_info().name() {
                                            "INT" | "TINYINT" | "SMALLINT" | "MEDIUMINT"
                                            | "BIGINT" => value
                                                .try_decode::<i64>()
                                                .map(|v| v.to_string())
                                                .unwrap_or("<err>".into()),
                                            "FLOAT" | "DOUBLE" | "DECIMAL" => value
                                                .try_decode::<f64>()
                                                .map(|v| v.to_string())
                                                .unwrap_or("<err>".into()),
                                            "VARCHAR" | "CHAR" | "TEXT" | "LONGTEXT" => value
                                                .try_decode::<String>()
                                                .unwrap_or("<err>".into()),
                                            "DATETIME" | "TIMESTAMP" => value
                                                .try_decode::<chrono::NaiveDateTime>()
                                                .map(|v| v.to_string())
                                                .unwrap_or("<err>".into()),
                                            _ => "<unsupported>".into(),
                                        }
                                    };

                                    records.push(value_str);
                                }

                                records
                            })
                            .fetch_all(&mut *tx)
                            .await?;

                        self.table_view.items = Some((columns, rows));
                    }
                    None => self.restore_widgets(),
                }
            }
        }

        Ok(())
    }
}

impl App {
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let (stop_tx, stop_rx) = watch::channel(false);

        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::task::spawn_blocking(move || {
            loop {
                if event::poll(Duration::from_millis(100)).unwrap()
                    && let Ok(event) = event::read()
                {
                    let _ = match event {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            tx.send(Event::Key(key))
                        }
                        Event::Resize(width, height) => tx.send(Event::Resize(width, height)),
                        _ => Ok(()),
                    };
                }

                if *stop_rx.borrow() {
                    break;
                }
            }
        });

        while !self.exit {
            tokio::select! {
                Some(event) = rx.recv() => {
                    match event {
                        Event::Key(key) => self.handle_key(key).await,
                        Event::Resize(_, _) => self.request_redraw = true,
                        _ => (),
                    };
                }
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                       self.request_update_data = true;
                       self.request_redraw = true;
                }
                _ = tokio::time::sleep(Duration::from_millis(50)) => {
                    if self.request_update_data {
                       self.update_data().await?;

                       self.request_update_data = false;
                    }

                    if self.request_redraw {
                        terminal.draw(|frame| self.draw(frame))?;

                        self.request_redraw = false;
                    }
                }
            }
        }

        let _ = stop_tx.send(true);

        Ok(())
    }
}
