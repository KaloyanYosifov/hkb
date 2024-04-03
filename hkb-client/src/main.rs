use crossterm::event::{self, Event, KeyCode};
use ratatui::{prelude::*, widgets::*};
use std::{io::Error as IOError, time::Duration};
use thiserror::Error as ThisError;

mod apps;
mod terminal;

#[derive(ThisError, Debug)]
pub enum RendererError {
    #[error("Failed to render output!")]
    FailedToRenderToOutput(#[from] IOError),
    #[error("Failed to initialize terminal")]
    FailedToInitializeTerminal(#[from] terminal::TerminalError),
}

type RenderResult = Result<(), RendererError>;

struct EventHandler {
    events: Vec<Event>,
}

impl EventHandler {
    fn new() -> Self {
        Self {
            // 10 is a random initial number here. We shouldn't be getting more than 10 events in one loop
            events: Vec::with_capacity(10),
        }
    }
}

impl EventHandler {
    fn all(&self) -> Vec<&Event> {
        self.events.iter().collect()
    }

    fn push(&mut self, event: Event) {
        self.events.push(event);
    }

    fn consume(&mut self, index: usize) {
        if self.events.get(index).is_none() {
            return;
        }

        self.events = self
            .events
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != index)
            .map(|(_, el)| el)
            .cloned()
            .collect();
    }

    fn clear(&mut self) {
        self.events.clear();
    }
}

fn main() -> RenderResult {
    let mut terminal = terminal::init()?;
    let mut should_quit = false;
    let mut main_app = apps::main::MainApp::new();
    let mut event_handler = EventHandler::new();

    terminal.clear()?;

    while !should_quit {
        // 30 FPS = 33 millis. Since poll is blocking we can simulate it as a sleep
        while event::poll(Duration::from_millis(33)).unwrap() {
            if let Ok(event) = event::read() {
                event_handler.push(event);
            }
        }

        terminal.draw(|frame| {
            should_quit = main_app.render(frame, &mut event_handler);
        })?;

        event_handler.clear();
    }

    terminal::close()?;

    Ok(())
}
