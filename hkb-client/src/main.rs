use crossterm::event;
use std::{io::Error as IOError, thread, time::Duration};
use thiserror::Error as ThisError;

mod apps;
mod components;
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

    terminal.clear()?;

    while !should_quit {
        // 30 FPS = 33 millis. Since poll is blocking we can simulate it as a sleep
        while event::poll(Duration::ZERO).unwrap() {
            if let Ok(event) = event::read() {
                events::push(event);
            }
        }

        terminal.draw(|frame| {
            should_quit = main_app.render(frame);
        })?;

        events::clear();

        thread::sleep(Duration::from_millis(33));
    }

    terminal::close()?;

    Ok(())
}
