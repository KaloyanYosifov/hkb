use crossterm::event::KeyCode;
use ratatui::text::Span;
use ratatui::{
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::events;
use crate::focus::Focusable;
use crate::utils::centered_layout;

use super::StatefulComponent;

pub struct ButtonState {
    clicked: bool,
    focused: bool,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            clicked: false,
            focused: false,
        }
    }
}

impl ButtonState {
    pub fn is_clicked(&self) -> bool {
        self.clicked
    }

    pub fn click(&mut self) {
        self.clicked = true;
    }

    pub fn unclick(&mut self) {
        self.clicked = false;
    }
}

impl Focusable for ButtonState {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }
}

pub struct Button<'a> {
    label: &'a str,
    centered: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            centered: false,
        }
    }

    pub fn center(mut self) -> Self {
        self.centered = true;

        self
    }
}

impl<'a> StatefulComponent for Button<'a> {
    type State = ButtonState;

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        state: &mut ButtonState,
        mut area: ratatui::prelude::Rect,
    ) {
        state.clicked = false;

        if state.focused && events::has_key_event!(KeyCode::Enter) {
            state.clicked = true;
        }

        let mut text = Span::raw(format!("<{}>", self.label));

        if state.focused {
            text = text.style(Style::default().bg(Color::Black).fg(Color::White));
        }

        let mut paragraph = Paragraph::new(text);

        if self.centered {
            area = centered_layout(area, 50, 50);
            paragraph = paragraph.centered();
        }

        frame.render_widget(paragraph, area);
    }
}
