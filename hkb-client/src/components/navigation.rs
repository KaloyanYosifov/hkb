use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::{Color, Constraint, Direction, Layout, Rect, Style},
    widgets::{Block, Paragraph, Tabs},
    Frame,
};

use crate::{events, AppState, AppView};

pub struct Navigation {
    title: String,
    views: Vec<AppView>,
    selected_tab: usize,
}

impl Navigation {
    pub fn new(title: String, views: Vec<AppView>) -> Self {
        Self {
            title,
            views,
            selected_tab: 0,
        }
    }
}

impl Navigation {
    pub fn render(&mut self, frame: &mut Frame, app_state: &mut AppState, area: Rect) {
        let tab_clicked = events::consume_if(|event| match event {
            Event::Key(event) => match event.code {
                KeyCode::Tab => true,
                _ => false,
            },
            _ => false,
        });

        if !tab_clicked.is_empty() {
            self.selected_tab += 1;

            if self.selected_tab >= self.views.len() {
                self.selected_tab = 0;
            }

            app_state.set_view(self.views[self.selected_tab].clone());
        }

        let block = Block::new().style(Style::default().bg(ratatui::style::Color::Black));
        let block_inner = block.inner(area);
        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        )
        .split(block_inner);
        let tabs = Tabs::new(self.views.iter().map(|view| view.to_string()))
            .select(self.selected_tab)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::White))
            .divider("")
            .padding_left(" ".repeat(3));

        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(self.title.clone()).style(Style::default().fg(Color::White)),
            layout[0],
        );
        frame.render_widget(tabs, layout[1]);
    }
}
