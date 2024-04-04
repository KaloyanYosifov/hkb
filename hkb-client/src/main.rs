use crossterm::event;
use std::{io::Error as IOError, time::Duration};
use thiserror::Error as ThisError;

mod apps;
mod events;
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
    let mut event_handler = events::EventHandler::new();

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
