use crossterm::event::{Event, KeyCode};
use hkb_core::logger::debug;
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
    cursor_offset: u16,
    visible_buffer_offset: usize,
    last_render_width: u16,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            focused: false,
            last_render_width: 0,
            cursor_offset: 0,
            visible_buffer_offset: 0,
            //buffer: String::with_capacity(512),
            buffer: String::from(
                "Testing this. If this is $ :) a good idea lorem. ipsum bopsum best test best mest sest Testing this if this is a good idea lorem ipsum bopsum best test best mest sest",
            ),
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
        if state.visible_buffer_offset >= (area.width as usize) {
            output = &state.buffer[state.visible_buffer_offset - (area.width as usize)..];
        }

        output
    }

    fn get_max_right_cursor_pos(&self, state: &InputState) -> u16 {
        std::cmp::min(state.buffer.len(), (state.last_render_width - 1) as usize) as u16
    }

    fn go_left(&self, state: &mut InputState) {
        if state.cursor_offset != 0 {
            state.cursor_offset -= 1;
        } else if state.visible_buffer_offset != 0 {
            state.visible_buffer_offset -= 1;
        }
    }

    fn go_right(&self, state: &mut InputState) {
        if state.cursor_offset < self.get_max_right_cursor_pos(state) {
            state.cursor_offset += 1;
        } else if state.visible_buffer_offset < state.buffer.len() {
            state.visible_buffer_offset += 1;
        }
    }

    fn go_far_left(&self, state: &mut InputState) {
        state.visible_buffer_offset = 0;
        state.cursor_offset = 0;
    }

    fn go_far_right(&self, state: &mut InputState) {
        state.visible_buffer_offset = state.buffer.len();
        state.cursor_offset = self.get_max_right_cursor_pos(state);
    }

    fn go_to_word<C: Fn(u16) -> bool, F: Fn(u16) -> u16>(
        &self,
        mut current_pos: usize,
        state: &mut InputState,
        condition: C,
        callback: F,
    ) {
        let chars = state.buffer.chars().collect::<Vec<char>>();

        // TODO: Should we support UTF8 instead of only ascii for inputs?
        while condition(current_pos as u16) && chars[current_pos].is_ascii_whitespace() {
            current_pos = callback(current_pos as u16) as usize;
        }

        if condition(current_pos as u16) && chars[current_pos].is_ascii_punctuation() {
            while condition(current_pos as u16) && chars[current_pos].is_ascii_punctuation() {
                current_pos = callback(current_pos as u16) as usize;
            }
        } else if condition(current_pos as u16) && chars[current_pos].is_ascii_alphanumeric() {
            while condition(current_pos as u16) && chars[current_pos].is_ascii_alphanumeric() {
                current_pos = callback(current_pos as u16) as usize;
            }
        }

        state.cursor_offset = std::cmp::min(
            current_pos.checked_sub(1).unwrap_or(0) as u16,
            self.get_max_right_cursor_pos(state),
        );
    }

    fn go_end_of_word(&self, state: &mut InputState) {
        let max_cursor_pos = self.get_max_right_cursor_pos(state);

        self.go_to_word(
            state.cursor_offset as usize + 1,
            state,
            |val| val < max_cursor_pos,
            |pos| pos + 1,
        );
    }

    fn go_back_word(&self, state: &mut InputState) {
        state.cursor_offset.checked_sub(1).unwrap_or(0) as usize;

        self.go_to_word(
            state.cursor_offset.checked_sub(1).unwrap_or(0) as usize,
            state,
            |val| val > 0,
            |pos| pos.checked_sub(1).unwrap_or(0),
        );
    }

    fn update_on_not_editing(&self, state: &mut InputState) {
        events::consume_key_event!(
            KeyCode::Char(c) => {
                match c {
                    'h' => self.go_left(state),
                    'l' => self.go_right(state),
                    '^' => self.go_far_left(state),
                    '$' => self.go_far_right(state),
                    'e' => self.go_end_of_word(state),
                    'b' => self.go_back_word(state),
                    _ => {}
                }
            }
        );
    }

    fn update(&self, state: &mut InputState) {
        if !app_state::is_editing() {
            self.update_on_not_editing(state);

            return;
        }

        events::consume_key_event!(
            KeyCode::Char(c) => {
                state.buffer.push(c);
                state.visible_buffer_offset = state.buffer.len();

                self.go_right(state);
            }
            KeyCode::Left => {
                self.go_left(state);
            }
            KeyCode::Right => {
                self.go_right(state);
            }
            KeyCode::Backspace => {
                if state.buffer.len() > 0 {
                    state.buffer = (&state.buffer[0..state.buffer.len() - 1]).to_owned();
                    state.visible_buffer_offset = state.buffer.len();

                    if state.cursor_offset != 0 {
                        state.cursor_offset -= 1;
                    }
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

            frame.set_cursor(state.cursor_offset + block_area.x, block_area.y);
        }

        state.last_render_width = block_area.width;
        frame.render_widget(
            Paragraph::new(self.trimmed_buffer(state, &area))
                .block(block.title(self.title.as_ref())),
            area,
        );
    }
}
