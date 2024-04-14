use crossterm::event::KeyCode;
use ratatui::prelude::{Constraint, Direction, Frame, Layout, Rect};
use ratatui::widgets::{Block, Borders, List};

use crate::{app_state, events};

use super::{Message, RemindersView};

pub struct RemindersList {}

impl Default for RemindersList {
    fn default() -> Self {
        Self {}
    }
}

impl RemindersView for RemindersList {
    fn init(&mut self) {
        app_state::set_editing(false);
        app_state::enable_navigation_events();
    }

    fn update(&mut self) -> Option<Message> {
        if events::has_key_event!(KeyCode::Char(c) if c == 'a' || c == 'A') {
            // self.view = View::Create;
            // app_state::set_editing(true);
            // app_state::disable_navigation_events();

            return Some(Message::ChangeView(super::View::Create));
        }

        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::new(
            Direction::Vertical,
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
}
