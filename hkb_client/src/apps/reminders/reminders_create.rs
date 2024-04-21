use crossterm::event::KeyCode;
use hkb_date::date::SimpleLocalDate;
use hkb_date::HumanDateParser;
use ratatui::prelude::{Constraint, Direction, Frame, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::components::{Button, ButtonState, Input, InputState, StatefulComponent};
use crate::utils::centered_layout;
use crate::{app_state, events, focus::Focusable};

use super::{CreateReminderData, Message, RemindersView};

pub struct RemindersCreate {
    error_message: Option<String>,
    focused_component: usize,
    message_input: InputState,
    reminder_date_input: InputState,
    submit_button: ButtonState,
    parsed_date: Option<SimpleLocalDate>,
}

impl Default for RemindersCreate {
    fn default() -> Self {
        Self {
            parsed_date: None,
            error_message: None,
            focused_component: 0,
            message_input: InputState::default(),
            reminder_date_input: InputState::default(),
            submit_button: ButtonState::default(),
        }
    }
}

// Create rendering
impl RemindersCreate {
    fn render_inputs(&mut self, frame: &mut Frame, area: Rect) {
        let input_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Length(3), Constraint::Length(3)],
        )
        .split(area);
        Input::new("Title").render(frame, &mut self.message_input, input_layout[0]);
        Input::new("Reminder Date").render(frame, &mut self.reminder_date_input, input_layout[1]);
    }

    fn render_error_messages(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(message) = self.error_message.as_ref() {
            frame.render_widget(
                Paragraph::new(message.clone())
                    .centered()
                    .style(Style::default().fg(Color::Red)),
                centered_layout(area, 50, 50),
            );
        }
    }

    fn render_create_button(&mut self, frame: &mut Frame, area: Rect) {
        Button::new("Create").center().render(
            frame,
            &mut self.submit_button,
            centered_layout(area, 50, 50),
        )
    }

    fn render_form(&mut self, frame: &mut Frame, area: Rect) {
        let has_error = self.error_message.is_some();
        let mut constraints: Vec<u16> = vec![80, 20];

        if has_error {
            constraints = vec![40, 40, 20];
        }

        let base_layout = Layout::new(
            Direction::Vertical,
            Constraint::from_percentages(constraints),
        )
        .split(area);

        self.render_inputs(frame, base_layout[0]);

        if has_error {
            self.render_error_messages(frame, base_layout[1]);
        }

        self.render_create_button(frame, base_layout[base_layout.len() - 1]);
    }

    fn validate(&mut self) -> bool {
        if self.message_input.buffer.len() <= 0 {
            self.error_message = Some("Title Input is required!".to_owned());
        } else if self.reminder_date_input.buffer.len() <= 0 {
            self.error_message = Some("Remidner Date Input is required!".to_owned());
        } else {
            let parser = HumanDateParser::new(SimpleLocalDate::now());

            match parser.parse(&self.reminder_date_input.buffer) {
                Ok(date) => {
                    self.parsed_date = Some(date);
                    self.error_message = None;

                    return true;
                }
                _ => self.error_message = Some("Failed to parse date!".to_owned()),
            }
        }

        false
    }
}

impl RemindersView for RemindersCreate {
    fn init(&mut self) {
        app_state::set_editing(true);
        app_state::disable_navigation_events();
    }

    fn update(&mut self) -> Option<Message> {
        if self.submit_button.is_clicked() {
            if self.validate() {
                let data = CreateReminderData {
                    message: self.message_input.buffer.to_owned(),
                    date: self.parsed_date.take().unwrap(),
                };
                return Some(Message::CreateReminder(data));
            }

            self.submit_button.unclick();
        }

        let mut focusables: Vec<&mut dyn Focusable> = vec![
            &mut self.message_input,
            &mut self.reminder_date_input,
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
            } else if events::has_key_event!(KeyCode::BackTab) {
                let last_focused_input = self.focused_component;

                if self.focused_component <= 0 {
                    self.focused_component = focusables.len() - 1;
                } else {
                    self.focused_component -= 1;
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
