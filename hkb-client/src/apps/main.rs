use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{Constraint, Direction, Frame, Layout};
use ratatui::widgets::{Block, Borders};

use crate::components::input::Input;
use crate::events;

pub struct MainApp {
    input: Input,
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            input: Input::new(),
        }
    }
}

impl MainApp {
    pub fn render(&mut self, frame: &mut Frame) -> bool {
        let should_quit_events = events::consume_if(|event| match event {
            Event::Key(event) => match event.code {
                KeyCode::Char(c) => {
                    c == 'c' && event.modifiers.contains(event::KeyModifiers::CONTROL)
                }
                _ => false,
            },
            _ => false,
        });

        // if we have quit events, then we quit
        if should_quit_events.len() > 0 {
            return true;
        }

        let main_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        )
        .split(frame.size());
        frame.render_widget(
            Block::new().borders(Borders::TOP).title("Title Bar"),
            main_layout[0],
        );
        frame.render_widget(
            Block::new().borders(Borders::TOP).title("Status Bar"),
            main_layout[2],
        );

        let inner_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(main_layout[1]);
        frame.render_widget(
            Block::default().borders(Borders::ALL).title("Left"),
            inner_layout[0],
        );

        let right_block = &Block::default().borders(Borders::ALL).title("Right");
        frame.render_widget(right_block, inner_layout[1]);

        self.input.render(frame, right_block.inner(inner_layout[1]));

        false
    }
}
