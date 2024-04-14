use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{app_state, events, focus::Focusable};

use super::StatefulComponent;

pub struct InputState {
    pub buffer: String,
    focused: bool,
    look_offset: usize,
    last_render_width: u16,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            focused: false,
            look_offset: 0,
            last_render_width: 0,
            buffer: String::with_capacity(512),
        }
    }
}

impl Focusable for InputState {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }
}

pub struct Input<'a> {
    title: &'a str,
}

impl<'a> Input<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title }
    }
}

impl<'a> Input<'a> {
    fn trimmed_buffer(&self, state: &'a InputState, area: &Rect) -> &str {
        let mut output = &state.buffer[..];
        if state.look_offset >= (area.width as usize) {
            output = &state.buffer[state.look_offset - (area.width as usize)..];
        }

        output
    }

    fn update(&self, state: &mut InputState) {
        if !app_state::is_editing() {
            return;
        }

        events::consume_key_event!(
            KeyCode::Char(c) => {
                state.buffer.push(c);
                state.look_offset = state.buffer.len();
            }
            KeyCode::Left => {
                if state.look_offset > state.last_render_width as usize {
                    state.look_offset -= 1;
                }
            }
            KeyCode::Right => {
                if state.look_offset < state.buffer.len() {
                    state.look_offset += 1;
                }
            }
            KeyCode::Backspace => {
                if state.buffer.len() > 0 {
                    state.buffer = (&state.buffer[0..state.buffer.len() - 1]).to_string();
                    state.look_offset = state.buffer.len();
                }
            }
        );
    }
}

impl<'a> StatefulComponent for Input<'a> {
    type State = InputState;

    fn render(&mut self, frame: &mut Frame, state: &mut InputState, area: Rect) {
        let block = Block::default().borders(Borders::ALL);
        let block_area = block.inner(area);

        if state.focused {
            self.update(state);

            let min_x = std::cmp::min(state.last_render_width as usize, state.look_offset) as u16;
            frame.set_cursor(min_x + block_area.x, block_area.y);
        }

        state.last_render_width = area.width;
        frame.render_widget(
            Paragraph::new(self.trimmed_buffer(state, &area))
                .block(block.title(self.title.as_ref())),
            area,
        );
    }
}
