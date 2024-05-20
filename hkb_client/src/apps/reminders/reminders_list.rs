use crossterm::event::KeyCode;
use hkb_core::database::services::reminders;
use hkb_core::database::services::reminders::{ReminderData, ReminderQueryOptions};
use hkb_core::logger::info;
use hkb_date::date::SimpleDate;
use hkb_date::duration::HumanizedDuration;
use ratatui::prelude::{Constraint, Direction, Frame, Layout, Rect};
use ratatui::widgets::{Block, Borders, List};

use crate::{app_state, events};

use super::{Message, RemindersView};

pub struct RemindersList {
    upcoming_reminders: Vec<ReminderData>,
    today_reminders: Vec<ReminderData>,
}

impl Default for RemindersList {
    fn default() -> Self {
        Self {
            upcoming_reminders: vec![],
            today_reminders: vec![],
        }
    }
}

impl RemindersList {
    fn format_reminder(&self, reminder: &ReminderData) -> String {
        let duration = {
            let duration = reminder.remind_at - SimpleDate::local();

            if duration.is_zero() {
                "already due".to_string()
            } else {
                duration.to_human_string()
            }
        };

        format!("{} - {}", reminder.note, duration)
    }

    fn render_reminders(
        &self,
        frame: &mut Frame,
        title: &str,
        reminders: &Vec<ReminderData>,
        area: Rect,
    ) {
        let notes = reminders
            .iter()
            .map(|reminder| self.format_reminder(reminder))
            .collect::<Vec<String>>();
        let list = List::new(notes).block(Block::default().borders(Borders::ALL).title(title));

        frame.render_widget(list, area);
    }
}

impl RemindersView for RemindersList {
    fn init(&mut self) {
        info!(target: "CLIENT_REMINDERS_LIST", "List reminders view initialized.");
        app_state::set_editing(false);
        app_state::enable_navigation_events();

        let end_date = SimpleDate::local().end_of_day().unwrap();
        let today_reminders_query_options = vec![ReminderQueryOptions::RemindAtBetween {
            end_date,
            start_date: SimpleDate::local().start_of_day().unwrap(),
        }];
        self.today_reminders =
            reminders::fetch_reminders(Some(today_reminders_query_options)).unwrap_or(vec![]);

        let rest_of_reminders_query_options =
            vec![ReminderQueryOptions::RemindAtGe { date: end_date }];
        self.upcoming_reminders =
            reminders::fetch_reminders(Some(rest_of_reminders_query_options)).unwrap_or(vec![]);
    }

    fn update(&mut self) -> Option<Message> {
        if events::has_key_event!(KeyCode::Char(c) if c == 'a' || c == 'A') {
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
        let vertical_split_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        )
        .split(layout[1]);

        self.render_reminders(
            frame,
            "Today Reminders",
            &self.today_reminders,
            vertical_split_layout[0],
        );
        self.render_reminders(
            frame,
            "Upcoming Reminders",
            &self.upcoming_reminders,
            vertical_split_layout[1],
        );
    }
}
