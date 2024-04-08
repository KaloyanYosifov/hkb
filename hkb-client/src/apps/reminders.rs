use crossterm::event::KeyCode;
use ratatui::prelude::Direction::Vertical;
use ratatui::prelude::{Constraint, Frame, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, Padding};

use crate::components::Input;
use crate::{app_state, events};

enum View {
    List,
    Create,
}

pub struct RemindersApp {
    view: View,
    inputs: Vec<Input>,
    focused_input: usize,
}

impl RemindersApp {
    pub fn new() -> Self {
        let mut inputs = vec![
            Input::new("Title".to_string()),
            Input::new("Small Description".to_string()),
        ];

        inputs[0].focus();

        Self {
            inputs,
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

                if self.focused_input >= self.inputs.len() {
                    self.focused_input = 0;
                }

                self.inputs[last_focused_input].blur();
                self.inputs[self.focused_input].focus();
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

        let input_layout =
            Layout::new(Vertical, [Constraint::Length(3), Constraint::Length(3)]).split(block_area);

        for (i, input) in self.inputs.iter_mut().enumerate() {
            input.render(frame, input_layout[i]);
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.view {
            View::List => self.render_list(frame, area),
            View::Create => self.render_create(frame, area),
        };
    }
}
