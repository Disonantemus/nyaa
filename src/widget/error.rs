use std::cmp::{max, min};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Rect,
    style::Stylize,
    widgets::{Paragraph, Widget as _, Wrap},
    Frame,
};

use crate::app::{App, Mode};

use super::{border_block, Widget};

pub struct ErrorPopup {
    pub error: String,
}

impl ErrorPopup {
    pub fn with_error(&mut self, error: String) {
        self.error = error;
    }
}

impl Default for ErrorPopup {
    fn default() -> Self {
        ErrorPopup {
            error: "".to_owned(),
        }
    }
}

impl Widget for ErrorPopup {
    fn draw(&self, f: &mut Frame, app: &App, area: Rect) {
        let lines = self.error.split('\n');
        let max_line = lines.clone().fold(30, |acc, e| max(e.len(), acc)) as u16 + 3;
        let x_len = min(max_line, area.width - 4);

        // Get number of lines including wrapped lines
        let height = lines.fold(0, |acc, e| {
            acc + (e.len() as f32 / (x_len - 2) as f32).ceil() as u16
        }) + 2;
        let center = super::centered_rect(x_len, height, area);
        let clear = super::centered_rect(center.width + 2, center.height, area);
        let p = Paragraph::new(self.error.to_owned())
            .block(
                border_block(app.theme, true)
                    .fg(app.theme.remake)
                    .title("Error: Press any key to dismiss"),
            )
            .wrap(Wrap { trim: false });
        super::clear(clear, f.buffer_mut(), app.theme.bg);
        p.render(center, f.buffer_mut());
    }

    fn handle_event(&mut self, app: &mut App, e: &Event) {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = e
        {
            match code {
                KeyCode::Esc | KeyCode::Char(_) => {
                    if app.errors.is_empty() {
                        app.mode = Mode::Normal;
                    }
                }
                _ => {}
            }
        }
    }

    fn get_help() -> Option<Vec<(&'static str, &'static str)>> {
        None
    }
}
