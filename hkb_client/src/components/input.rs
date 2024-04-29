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

    fn get_char_class(&self, pos: usize, chars: &Vec<char>) -> i8 {
        if pos >= chars.len() {
            return -1;
        }

        if chars[pos].is_whitespace() {
            0
        } else if chars[pos].is_ascii_punctuation() {
            1
        } else {
            2
        }
    }

    fn go_end_of_word(&self, state: &mut InputState) {
        let chars = state.buffer.chars().collect::<Vec<char>>();
        let mut current_pos = state.cursor_offset as usize + 1;

        while self.get_char_class(current_pos, &chars) == 0 {
            current_pos += 1;
        }

        let char_class = self.get_char_class(current_pos, &chars);

        if char_class != -1 {
            while self.get_char_class(current_pos, &chars) == char_class {
                current_pos += 1;
            }
        }

        state.cursor_offset = std::cmp::min(
            current_pos.checked_sub(1).unwrap_or(0) as u16,
            self.get_max_right_cursor_pos(state),
        );
    }

    fn go_back_word(&self, state: &mut InputState) {
        let chars = state.buffer.chars().collect::<Vec<char>>();
        let mut current_pos = state.cursor_offset.checked_sub(1).unwrap_or(0) as usize;

        while self.get_char_class(current_pos, &chars) == 0 {
            current_pos = current_pos.checked_sub(1).unwrap_or(0);

            if current_pos == 0 {
                break;
            }
        }

        let char_class = self.get_char_class(current_pos, &chars);

        if current_pos != 0 && char_class != -1 {
            while self.get_char_class(current_pos, &chars) == char_class {
                current_pos = current_pos.checked_sub(1).unwrap_or(0);

                if current_pos == 0 {
                    break;
                }
            }
        }

        if current_pos == 0 {
            state.cursor_offset = 0;
        } else {
            state.cursor_offset = (current_pos + 1) as u16;
        }
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
                // TODO: make it work inside the view range of the string
                let offset = state.cursor_offset as usize;
                let first_part = &state.buffer[..offset];
                let second_part = &state.buffer[offset..];
                let mut buffer = String::with_capacity(first_part.len() + second_part.len() + 1);

                buffer.push_str(first_part);
                buffer.push(c);
                buffer.push_str(second_part);

                state.buffer = buffer;
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
