use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Row, StatefulWidget as _, Table},
    Frame,
};
use serde::{Deserialize, Serialize};

use crate::app::{App, LoadType, Mode};

use super::{border_block, EnumIter, StatefulTable, Widget};

#[derive(Clone, Serialize, Deserialize)]
pub enum Sort {
    Date,
    Downloads,
    Seeders,
    Leechers,
    Size,
}

#[derive(PartialEq, Clone)]
pub enum SortDir {
    Desc,
    Asc,
}

impl EnumIter<Sort> for Sort {
    fn iter() -> std::slice::Iter<'static, Sort> {
        static SORTS: &[Sort] = &[
            Sort::Date,
            Sort::Downloads,
            Sort::Seeders,
            Sort::Leechers,
            Sort::Size,
        ];
        SORTS.iter()
    }
}

impl ToString for Sort {
    fn to_string(&self) -> String {
        match self {
            Sort::Date => "Date".to_owned(),
            Sort::Downloads => "Downloads".to_owned(),
            Sort::Seeders => "Seeders".to_owned(),
            Sort::Leechers => "Leechers".to_owned(),
            Sort::Size => "Size".to_owned(),
        }
    }
}

impl Sort {
    pub fn to_url(&self) -> String {
        match self {
            Sort::Date => "id".to_owned(),
            Sort::Downloads => "downloads".to_owned(),
            Sort::Seeders => "seeders".to_owned(),
            Sort::Leechers => "leechers".to_owned(),
            Sort::Size => "size".to_owned(),
        }
    }
}

pub struct SortPopup {
    pub table: StatefulTable<String>,
    pub selected: Sort,
}

impl Default for SortPopup {
    fn default() -> Self {
        SortPopup {
            table: StatefulTable::with_items(Sort::iter().map(|item| item.to_string()).collect()),
            selected: Sort::Date,
        }
    }
}

impl Widget for SortPopup {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let buf = f.buffer_mut();
        let center = super::centered_rect(30, self.table.items.len() as u16 + 2, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let items = self.table.items.iter().enumerate().map(|(i, item)| {
            Row::new(vec![match i == self.selected.to_owned() as usize {
                true => format!("  {}", item.to_owned()),
                false => format!("   {}", item.to_owned()),
            }])
        });
        let table = Table::new(items, [Constraint::Percentage(100)])
            .block(border_block(app.theme, true).title(
                match app.mode == Mode::Sort(SortDir::Asc) {
                    true => "Sort Ascending",
                    false => "Sort Descending",
                },
            ))
            .highlight_style(Style::default().bg(app.theme.hl_bg));
        super::clear(clear, buf, app.theme.bg);
        table.render(center, buf, &mut self.table.state.to_owned());
    }

    fn handle_event(&mut self, app: &mut crate::app::App, e: &crossterm::event::Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('q') => {
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.table.next_wrap(1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.table.next_wrap(-1);
                }
                KeyCode::Char('G') => {
                    self.table.select(self.table.items.len() - 1);
                }
                KeyCode::Char('g') => {
                    self.table.select(0);
                }
                KeyCode::Enter => {
                    if let Some(i) = Sort::iter().nth(self.table.state.selected().unwrap_or(0)) {
                        self.selected = i.to_owned();
                        app.ascending = app.mode == Mode::Sort(SortDir::Asc);
                        app.mode = Mode::Loading(LoadType::Sorting);
                    }
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        Some(vec![
            ("Enter", "Confirm"),
            ("Esc, s, q", "Close"),
            ("j, ↓", "Down"),
            ("k, ↑", "Up"),
            ("g", "Top"),
            ("G", "Bottom"),
        ])
    }
}
