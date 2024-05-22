use crossterm::event::KeyCode;
use hkb_core::database::services::reminders;
use hkb_core::database::services::reminders::{ReminderData, ReminderQueryOptions};
use hkb_core::logger::info;
use hkb_date::date::SimpleDate;
use hkb_date::duration::HumanizedDuration;
use ratatui::prelude::{Constraint, Direction, Frame, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListState};

use crate::utils::bounded_value::BoundedValue;
use crate::{app_state, events};

use super::{Message, RemindersView};

pub struct RemindersList {
    selected: BoundedValue,

    today_reminders: Vec<ReminderData>,
    today_reminders_state: ListState,

    upcoming_reminders: Vec<ReminderData>,
    upcoming_reminders_state: ListState,
}

impl Default for RemindersList {
    fn default() -> Self {
        Self {
            selected: BoundedValue::new(0, 0, 0),

            today_reminders: vec![],
            today_reminders_state: ListState::default().with_selected(Some(0)),

            upcoming_reminders: vec![],
            upcoming_reminders_state: ListState::default().with_selected(None),
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

    fn create_reminder_list<'a>(&self, reminders: &Vec<ReminderData>, title: &'a str) -> List<'a> {
        let notes = reminders
            .iter()
            .map(|reminder| self.format_reminder(reminder))
            .collect::<Vec<String>>();

        List::new(notes)
            .highlight_style(
                Style::default()
                    .bg(ratatui::style::Color::Black)
                    .fg(ratatui::style::Color::White)
                    .add_modifier(Modifier::ITALIC),
            )
            .block(Block::default().borders(Borders::ALL).title(title))
    }

    fn update_selected_reminder(&mut self) {
        if events::has_key_event!(KeyCode::Char(c) if c == 'j') {
            self.selected += 1;
        } else if events::has_key_event!(KeyCode::Char(c) if c == 'k') {
            self.selected -= 1;
        }

        if self.selected >= self.today_reminders.len() {
            let upcoming_selected = self.selected.get_val() - self.today_reminders.len();

            if upcoming_selected < self.upcoming_reminders.len() {
                self.today_reminders_state.select(None);
                self.upcoming_reminders_state
                    .select(Some(upcoming_selected));
            }
        } else {
            self.upcoming_reminders_state.select(None);
            self.today_reminders_state
                .select(Some(self.selected.get_val()));
        }
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

        self.selected.set_max(
            (self.today_reminders.len() + self.upcoming_reminders.len())
                .checked_sub(1)
                .unwrap_or(0),
        );
    }

    fn update(&mut self) -> Option<Message> {
        if events::has_key_event!(KeyCode::Char(c) if c == 'a' || c == 'A') {
            return Some(Message::ChangeView(super::View::Create));
        }

        if events::has_key_event!(KeyCode::Backspace)
            || events::has_key_event!(KeyCode::Char(c) if c == 'd')
        {
            let reminder = {
                if self.selected >= self.today_reminders.len() {
                    self.upcoming_reminders
                        .get(self.selected.get_val() - self.today_reminders.len())
                        .unwrap()
                } else {
                    self.today_reminders.get(self.selected.get_val()).unwrap()
                }
            };

            return Some(Message::DeleteReminder(reminder.id));
        }

        self.update_selected_reminder();

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

        let list = self.create_reminder_list(&self.today_reminders, "Today Reminders");
        frame.render_stateful_widget(
            list,
            vertical_split_layout[0],
            &mut self.today_reminders_state,
        );

        let list = self.create_reminder_list(&self.upcoming_reminders, "Upcoming Reminders");
        frame.render_stateful_widget(
            list,
            vertical_split_layout[1],
            &mut self.upcoming_reminders_state,
        );
    }
}
