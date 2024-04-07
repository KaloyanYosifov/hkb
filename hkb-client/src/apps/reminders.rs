use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::Paragraph;

use crate::AppState;

pub struct RemindersApp {}

impl RemindersApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl RemindersApp {
    pub fn render(&mut self, frame: &mut Frame, app_state: &mut AppState, area: Rect) {
        frame.render_widget(Paragraph::new("Reminders"), area);
    }
}
