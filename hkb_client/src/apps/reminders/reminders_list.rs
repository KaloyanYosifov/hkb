use crossterm::event::KeyCode;
use hkb_core::database::services;
use hkb_core::database::services::reminders::ReminderData;
use hkb_core::logger::info;
use ratatui::prelude::{Constraint, Direction, Frame, Layout, Rect};
use ratatui::widgets::{Block, Borders, List};

use crate::{app_state, events};

use super::{Message, RemindersView};

pub struct RemindersList {
    reminders: Vec<ReminderData>,
}

impl Default for RemindersList {
    fn default() -> Self {
        Self { reminders: vec![] }
    }
}

impl RemindersView for RemindersList {
    fn init(&mut self) {
        info!(target: "CLIENT_REMINDERS_LIST", "List reminders view initialized.");
        app_state::set_editing(false);
        app_state::enable_navigation_events();
        self.reminders = services::reminders::fetch_reminders(None).unwrap_or(vec![]);
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
        let reminder_notes = self
            .reminders
            .iter()
            .map(|reminder| reminder.note.as_str())
            .collect::<Vec<&str>>();
        let list = List::new(reminder_notes)
            .block(Block::default().borders(Borders::ALL).title("Reminders"));

        frame.render_widget(list, layout[1]);
    }
}
