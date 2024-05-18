use crossterm::event::{Event, KeyCode};
use ratatui::{
    prelude::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app_state, events,
    focus::Focusable,
    utils::bounded_value::{BoundValueType, BoundedValue},
};

use super::StatefulComponent;

pub struct InputState {
    pub buffer: String,
    focused: bool,
    cursor_offset: BoundedValue,
    visible_buffer_offset: usize,
    last_render_width: u16,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            focused: false,
            last_render_width: 0,
            cursor_offset: BoundedValue::new(0, 0, 0),
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
        let area_width = area.width as usize;
        let offset_end = std::cmp::min(
            state.buffer.len(),
            (area_width as usize) + state.visible_buffer_offset,
        );

        &state.buffer[state.visible_buffer_offset..offset_end]
    }

    fn get_max_right_cursor_pos(&self, state: &InputState) -> BoundValueType {
        std::cmp::min(
            state.buffer.len(),
            (state.last_render_width - 1) as BoundValueType,
        )
    }

    fn go_left(&self, state: &mut InputState) {
        if state.cursor_offset != 0 {
            state.cursor_offset -= 1;
        } else if state.visible_buffer_offset != 0 {
            state.visible_buffer_offset -= 1;
        }
    }

    fn go_right(&self, state: &mut InputState) {
        let visible_buffer_len = (state.last_render_width as usize) + state.visible_buffer_offset;
        let max_right_pos = self.get_max_right_cursor_pos(state);

        if state.cursor_offset.lt(max_right_pos) {
            state.cursor_offset.set_max(max_right_pos);
            state.cursor_offset += 1;
        } else if visible_buffer_len < state.buffer.len() {
            state.visible_buffer_offset += 1;
        }
    }

    fn go_far_left(&self, state: &mut InputState) {
        state.visible_buffer_offset = 0;
        state.cursor_offset.set_val(0);
    }

    fn go_far_right(&self, state: &mut InputState) {
        let buffer_len = state.buffer.len();
        state
            .cursor_offset
            .set_val(self.get_max_right_cursor_pos(state));

        state.visible_buffer_offset = buffer_len
            .checked_sub(state.last_render_width as usize)
            .unwrap_or(0);
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
        if state.buffer.len() == 0 {
            return;
        }

        let chars = state.buffer.chars().collect::<Vec<char>>();
        let mut current_pos = self.get_buffer_update_offset(state) + 1;

        while self.get_char_class(current_pos, &chars) == 0 {
            current_pos += 1;
        }

        let char_class = self.get_char_class(current_pos, &chars);

        if char_class != -1 {
            while self.get_char_class(current_pos, &chars) == char_class {
                current_pos += 1;
            }
        }

        state.cursor_offset.set_val(std::cmp::min(
            current_pos.checked_sub(1).unwrap_or(0),
            self.get_max_right_cursor_pos(state),
        ));
        let visible_current_pos = current_pos - state.visible_buffer_offset;

        if visible_current_pos >= self.get_max_right_cursor_pos(state) - 1 {
            state.visible_buffer_offset = current_pos - state.cursor_offset.get_val() - 1;
        }
    }

    fn go_back_word(&self, state: &mut InputState) {
        if state.buffer.len() == 0 {
            return;
        }

        let chars = state.buffer.chars().collect::<Vec<char>>();
        let mut current_pos = self
            .get_buffer_update_offset(state)
            .checked_sub(1)
            .unwrap_or(0);

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

        if state.visible_buffer_offset > current_pos {
            state.visible_buffer_offset -= current_pos;
            state.cursor_offset.set_to_min();
        } else {
            // TOOD: check why back word is not working correctly
            // when we have text longer than the visible input
            state
                .cursor_offset
                .set_val(current_pos - state.visible_buffer_offset);
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
                };

                if ! state.focused {
                    return;
                }

                let should_edit = match c {
                    'i' => true,
                    'I' => {
                        self.go_far_left(state);

                        true
                    },
                    'a' => {
                        state.cursor_offset += 1;

                        true
                    },
                    'A' => {
                        self.go_far_right(state);

                        true
                    },
                    _ => false,
                };

                app_state::set_editing(should_edit);
            }
        );
    }

    fn get_buffer_update_offset(&self, state: &InputState) -> usize {
        state.visible_buffer_offset + state.cursor_offset.get_val()
    }

    fn on_char(&self, c: char, state: &mut InputState) {
        let offset = self.get_buffer_update_offset(state);
        let first_part = &state.buffer[..offset];
        let second_part = &state.buffer[offset..];
        let mut buffer = String::with_capacity(first_part.len() + second_part.len() + 1);

        buffer.push_str(first_part);
        buffer.push(c);
        buffer.push_str(second_part);

        state.buffer = buffer;

        if (state.cursor_offset.get_val() + 1) >= state.last_render_width as BoundValueType {
            state.visible_buffer_offset += 1;
        }

        self.go_right(state);
    }

    fn on_backspace(&self, state: &mut InputState) {
        if state.buffer.len() == 0 {
            return;
        }

        let offset = self.get_buffer_update_offset(state);
        let first_part = &state.buffer[..(offset.checked_sub(1).unwrap_or(0))];
        let second_part = &state.buffer[offset..];
        let buffer_capacity = (first_part.len() + second_part.len())
            .checked_sub(1)
            .unwrap_or(0);
        let mut buffer = String::with_capacity(buffer_capacity);

        buffer.push_str(first_part);
        buffer.push_str(second_part);

        state.buffer = buffer;
        state.cursor_offset.sub_val(1);
    }

    fn update(&self, state: &mut InputState) {
        state
            .cursor_offset
            .set_max(self.get_max_right_cursor_pos(state));

        if !app_state::is_editing() {
            self.update_on_not_editing(state);

            return;
        }

        events::consume_key_event!(
            KeyCode::Char(c) => {
                self.on_char(c, state);
            }
            KeyCode::Left => {
                self.go_left(state);
            }
            KeyCode::Right => {
                self.go_right(state);
            }
            KeyCode::Backspace => {
                self.on_backspace(state);
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

            frame.set_cursor(
                state.cursor_offset.get_val() as u16 + block_area.x,
                block_area.y,
            );
        }

        state.last_render_width = block_area.width;
        frame.render_widget(
            Paragraph::new(self.trimmed_buffer(state, &area))
                .block(block.title(self.title.as_ref())),
            area,
        );
    }
}
