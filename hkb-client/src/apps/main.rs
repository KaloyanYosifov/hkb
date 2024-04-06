use std::cell::RefCell;
use std::rc::Rc;

use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{Constraint, Direction, Frame, Layout};
use ratatui::widgets::{Block, Borders};

use crate::components::input::Input;
use crate::events;
use crate::focus::FocusHandler;

pub struct MainApp<'a> {
    left_input: Rc<RefCell<Input>>,
    right_input: Rc<RefCell<Input>>,
    focus_handler: FocusHandler<'a>,
}

impl<'a> MainApp<'a> {
    pub fn new() -> Self {
        let left_input = Rc::new(RefCell::new(Input::new()));
        let right_input = Rc::new(RefCell::new(Input::new()));
        let mut focus_handler = FocusHandler::new(2);

        focus_handler.add(Rc::clone(&left_input));
        focus_handler.add(Rc::clone(&right_input));

        Self {
            left_input,
            right_input,
            focus_handler,
        }
    }
}

impl<'a> MainApp<'a> {
    fn update(&mut self) {
        self.focus_handler.update();
    }

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

        self.update();

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

        let left_block = &Block::default().borders(Borders::ALL).title("Right");
        frame.render_widget(left_block, inner_layout[0]);

        (*self.left_input.borrow_mut()).render(frame, left_block.inner(inner_layout[0]));

        let right_block = &Block::default().borders(Borders::ALL).title("Right");
        frame.render_widget(right_block, inner_layout[1]);

        (*self.right_input.borrow_mut()).render(frame, right_block.inner(inner_layout[1]));

        false
    }
}
