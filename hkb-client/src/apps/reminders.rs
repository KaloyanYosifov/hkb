use crossterm::event::KeyCode;
use ratatui::prelude::Direction::Vertical;
use ratatui::prelude::{Constraint, Frame, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, Padding};

use crate::components::{Input, InputState, StatefulComponent};
use crate::{app_state, events};

enum View {
    List,
    Create,
}

pub struct RemindersApp {
    view: View,
    focused_input: usize,
    input_states: Vec<InputState>,
}

impl RemindersApp {
    pub fn new() -> Self {
        let mut input_states = vec![InputState::default(), InputState::default()];

        input_states[0].focus();

        Self {
            input_states,
            view: View::List,
            focused_input: 0,
        }
    }
}

impl RemindersApp {
    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        if events::has_key_event!(KeyCode::Char(c) if c == 'a' || c == 'A') {
            self.view = View::Create;
            app_state::set_editing(true);
            app_state::disable_navigation_events();

            return;
        }

        let layout = Layout::new(
            Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        )
        .split(area);
        let list = List::new(vec!["Item 1", "Item 2", "Item 3"])
            .block(Block::default().borders(Borders::ALL).title("Reminders"));

        frame.render_widget(list, layout[1]);
    }

    fn render_create(&mut self, frame: &mut Frame, area: Rect) {
        if !app_state::is_editing() {
            if events::has_key_event!(KeyCode::Tab) {
                let last_focused_input = self.focused_input;
                self.focused_input += 1;

                if self.focused_input >= self.input_states.len() {
                    self.focused_input = 0;
                }

                self.input_states[last_focused_input].blur();
                self.input_states[self.focused_input].focus();
            } else if events::has_key_event!(KeyCode::Char(c) if c == 'I' || c == 'i' || c == 'A' || c == 'a')
            {
                app_state::set_editing(true);
            }
        }

        let layout = Layout::new(
            Vertical,
            [
                Constraint::Percentage(25),
                Constraint::Min(0),
                Constraint::Length(25),
            ],
        )
        .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::symmetric(1, 1))
            .title("Create Reminder");
        let block_area = block.inner(layout[1]);
        frame.render_widget(block, layout[1]);

        let component_layout =
            Layout::new(Vertical, [Constraint::Length(3), Constraint::Length(3)]).split(block_area);

        Input::new("Title").render(frame, &mut self.input_states[0], component_layout[0]);
        Input::new("Small Description").render(
            frame,
            &mut self.input_states[1],
            component_layout[1],
        );
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.view {
            View::List => self.render_list(frame, area),
            View::Create => self.render_create(frame, area),
        };
    }
}
