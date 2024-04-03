use ratatui::prelude::{Constraint, Direction, Frame, Layout};
use ratatui::widgets::{Block, Borders};

pub struct MainApp {}

impl MainApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl MainApp {
    pub fn render(&mut self, frame: &mut Frame) {
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
        frame.render_widget(
            Block::default().borders(Borders::ALL).title("Right"),
            inner_layout[1],
        );
    }
}
