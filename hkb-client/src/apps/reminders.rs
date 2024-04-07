use ratatui::prelude::Direction::Vertical;
use ratatui::prelude::{Constraint, Frame, Layout, Rect};
use ratatui::widgets::{Block, Borders, List};

use crate::components::Input;

enum View {
    List,
    Create,
}

pub struct RemindersApp {
    view: View,
    inputs: Vec<Input>,
}

impl RemindersApp {
    pub fn new() -> Self {
        Self {
            view: View::List,
            inputs: vec![Input::new(), Input::new()],
        }
    }
}

impl RemindersApp {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.view {
            View::List => {
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
            View::Create => {
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
                    .title("Create Reminder");
                let block_area = block.inner(layout[1]);
                frame.render_widget(block, layout[1]);

                let input_layout =
                    Layout::new(Vertical, [Constraint::Length(2), Constraint::Length(2)])
                        .split(block_area);

                for (i, input) in self.inputs.iter_mut().enumerate() {
                    input.render(frame, input_layout[i]);
                }
            }
        };
    }
}
