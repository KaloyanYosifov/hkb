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

fn main() -> RenderResult {
    let mut terminal = terminal::init()?;
    let mut should_quit = false;
    let mut main_app = apps::main::MainApp::new();

    terminal.clear()?;

    while !should_quit {
        // 30 FPS = 33 millis. Since poll is blocking we can simulate it as a sleep
        while event::poll(Duration::from_millis(33)).unwrap() {
            match event::read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(c) => {
                        if c == 'c' && event.modifiers.contains(event::KeyModifiers::CONTROL) {
                            should_quit = true;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        terminal.draw(|frame| {
            main_app.render(frame);
        })?;
    }

    terminal::close()?;

    Ok(())
}
