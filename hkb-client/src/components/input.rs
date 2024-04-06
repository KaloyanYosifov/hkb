use crossterm::event::{Event, KeyCode};
use ratatui::{prelude::Rect, widgets::Paragraph, Frame};

use crate::{events, focus::Focusable};

pub struct Input {
    buffer: String,
    look_offset: usize,
    last_render_width: u16,
    focused: bool,
}

impl Focusable for Input {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }
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

    fn update(&mut self) {
        let key_pressed_events = events::consume_if(|event| match event {
            Event::Key(event) => match event.code {
                KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Left | KeyCode::Right => true,
                _ => false,
            },
            _ => false,
        });

        for event in key_pressed_events {
            match event {
                Event::Key(key) => match key.code {
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
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.focused {
            self.update();

            let min_x = std::cmp::min(self.last_render_width as usize, self.look_offset + 1) as u16;
            frame.set_cursor(min_x, area.y);
        }

        self.last_render_width = area.width;
        frame.render_widget(Paragraph::new(self.trimmed_buffer(&area)), area);
    }
}
