use hkb_core::logger::{debug, info};
use hkb_date::date::SimpleLocalDate;
use ratatui::prelude::{Frame, Rect};

use self::reminders_create::RemindersCreate;
use self::reminders_list::RemindersList;

mod reminders_create;
mod reminders_list;

trait RemindersView {
    fn init(&mut self);
    fn update(&mut self) -> Option<Message>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

enum View {
    List,
    Create,
}

impl Into<Box<dyn RemindersView>> for View {
    fn into(self) -> Box<dyn RemindersView> {
        match self {
            Self::List => Box::new(RemindersList::default()),
            Self::Create => Box::new(RemindersCreate::default()),
        }
    }
}

#[derive(Debug)]
struct CreateReminderData {
    message: String,
    date: SimpleLocalDate,
}

enum Message {
    ChangeView(View),
    CreateReminder(CreateReminderData),
}

pub struct RemindersApp {
    current_view: Box<dyn RemindersView>,
}

impl RemindersApp {
    pub fn new() -> Self {
        let mut current_view: Box<dyn RemindersView> = View::List.into();
        current_view.init();

        Self { current_view }
    }
}

impl RemindersApp {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.current_view.update() {
            Some(m) => match m {
                Message::ChangeView(view) => {
                    self.current_view = view.into();
                    self.current_view.init();
                }
                Message::CreateReminder(message) => {
                    info!(target: "REMINDERS", "Creating a reminder.");
                    debug!(target: "REMINDERS", "Received a message to create a reminder with {message:?}");

                    self.current_view = View::List.into();
                    self.current_view.init();
                }
            },
            _ => {}
        };

        self.current_view.render(frame, area);
    }
}
