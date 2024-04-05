use crossterm::event::{Event, KeyCode};
use ratatui::{prelude::Rect, widgets::Paragraph, Frame};

use crate::events;

pub struct Input {
    buffer: String,
}

impl Input {
    pub fn new() -> Self {
        Self {
            buffer: String::with_capacity(512),
        }
    }
}

impl Input {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let key_pressed_events = events::consume_if(|event| match event {
            Event::Key(event) => match event.code {
                KeyCode::Char(_) | KeyCode::Backspace => true,
                _ => false,
            },
            _ => false,
        });

        for event in key_pressed_events {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char(c) => {
                        self.buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        if self.buffer.len() > 0 {
                            self.buffer = (&self.buffer[0..self.buffer.len() - 1]).to_string();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        frame.render_widget(Paragraph::new(self.buffer.as_str()), area);
    }
}
