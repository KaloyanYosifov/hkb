use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::events;

pub struct Input {
    buffer: String,
    look_offset: usize,
    last_render_width: u16,
    focused: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            focused: false,
            look_offset: 0,
            last_render_width: 0,
            buffer: String::with_capacity(512),
        }
    }
}

impl Input {
    fn trimmed_buffer(&self, area: &Rect) -> &str {
        let mut output = &self.buffer[..];
        if self.look_offset >= (area.width as usize) {
            output = &self.buffer[self.look_offset - (area.width as usize)..];
        }

        output
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn blur(&mut self) {
        self.focused = false;
    }

    fn update(&mut self) {
        events::consume_key_event!(
            KeyCode::Char(c) => {
                self.buffer.push(c);
                self.look_offset = self.buffer.len();
            }
            KeyCode::Left => {
                if self.look_offset > self.last_render_width as usize {
                    self.look_offset -= 1;
                }
            }
            KeyCode::Right => {
                if self.look_offset < self.buffer.len() {
                    self.look_offset += 1;
                }
            }
            KeyCode::Backspace => {
                if self.buffer.len() > 0 {
                    self.buffer = (&self.buffer[0..self.buffer.len() - 1]).to_string();
                    self.look_offset = self.buffer.len();
                }
            }
        );
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.focused {
            self.update();

            let min_x = std::cmp::min(self.last_render_width as usize, self.look_offset + 1) as u16;
            frame.set_cursor(min_x, area.y);
        }

        self.last_render_width = area.width;
        frame.render_widget(
            Paragraph::new(self.trimmed_buffer(&area))
                .block(Block::default().title("bro").borders(Borders::ALL)),
            area,
        );
    }
}
