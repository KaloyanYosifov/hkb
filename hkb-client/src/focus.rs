use std::{cell::RefCell, rc::Rc};

use crossterm::event::{Event, KeyCode};

use crate::events;

pub trait Focusable {
    fn focus(&mut self);
    fn blur(&mut self);
}

pub struct FocusHandler<'a> {
    focus_id: usize,
    elements: Vec<Rc<RefCell<dyn Focusable + 'a>>>,
}

impl<'a> FocusHandler<'a> {
    pub fn new(capacity: usize) -> Self {
        Self {
            focus_id: 0,
            elements: Vec::with_capacity(capacity),
        }
    }
}

impl<'a> FocusHandler<'a> {
    pub fn add(&mut self, element: Rc<RefCell<impl Focusable + 'a>>) {
        if self.elements.len() == 0 {
            (*element.borrow_mut()).focus();
        }
        self.elements.push(element);
    }

    pub fn update(&mut self) {
        // if elements is empty
        // we do not do anything
        if self.elements.is_empty() {
            return;
        }

        let events = events::consume_if(|event| match event {
            Event::Key(event) => match event.code {
                KeyCode::Tab => true,
                _ => false,
            },
            _ => false,
        });

        if events.is_empty() {
            return;
        }

        let last_focused = self.focus_id;
        self.focus_id += 1;

        if self.focus_id >= self.elements.len() {
            self.focus_id = 0;
        }

        (*self.elements[last_focused].borrow_mut()).blur();
        (*self.elements[self.focus_id].borrow_mut()).focus();
    }
}
