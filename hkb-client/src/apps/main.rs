use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::Paragraph;

pub struct MainApp {}

impl MainApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl MainApp {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("hehehe"), area);
    }
}
