use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Frame, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, Padding};

use crate::components::{Button, ButtonState, Input, InputState, StatefulComponent};
use crate::{app_state, events, focus::Focusable};

use super::{Message, RemindersView};

pub struct RemindersCreate {
    focused_component: usize,
    title_input: InputState,
    description_input: InputState,
    submit_button: ButtonState,
}

impl Default for RemindersCreate {
    fn default() -> Self {
        Self {
            focused_component: 0,
            title_input: InputState::default(),
            description_input: InputState::default(),
            submit_button: ButtonState::default(),
        }
    }
}

// Create rendering
impl RemindersCreate {
    fn render_form(&mut self, frame: &mut Frame, area: Rect) {
        let base_layout =
            Layout::new(Direction::Vertical, Constraint::from_percentages([80, 20])).split(area);
        let input_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Length(3)],
        )
        .split(base_layout[0]);
        let button_layout = Layout::new(Direction::Horizontal, Constraint::from_mins([0, 100, 0]))
            .split(base_layout[1]);

        Input::new("Title").render(frame, &mut self.title_input, input_layout[0]);
        Input::new("Small Description").render(frame, &mut self.description_input, input_layout[1]);
        Button::new("Create")
            .center()
            .render(frame, &mut self.submit_button, button_layout[1])
    }
}

impl RemindersView for RemindersCreate {
    fn init(&mut self) {
        app_state::set_editing(true);
        app_state::disable_navigation_events();
    }

    fn update(&mut self) -> Option<Message> {
        if self.submit_button.is_clicked() {
            return Some(Message::ChangeView(super::View::List));
        }

        let mut focusables: Vec<&mut dyn Focusable> = vec![
            &mut self.title_input,
            &mut self.description_input,
            &mut self.submit_button,
        ];

        if !app_state::is_editing() {
            if events::has_key_event!(KeyCode::Tab) {
                let last_focused_input = self.focused_component;
                self.focused_component += 1;

                if self.focused_component >= focusables.len() {
                    self.focused_component = 0;
                }

                focusables[last_focused_input].blur();
            } else if events::has_key_event!(KeyCode::Char(c) if c == 'I' || c == 'i' || c == 'A' || c == 'a')
            {
                app_state::set_editing(true);
            }
        }

        focusables[self.focused_component].focus();

        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(30),
                Constraint::Min(0),
                Constraint::Percentage(30),
            ],
        )
        .split(area);
        let block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::symmetric(1, 1))
            .title("Create a Reminder");
        let block_area = block.inner(layout[1]);

        frame.render_widget(block, layout[1]);
        self.render_form(frame, block_area);
    }
}
