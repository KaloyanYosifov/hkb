use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{self as crossterminal},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, style::Stylize, widgets::Paragraph, Terminal};
use std::panic;
use std::{
    io::{stdout, Error as IOError},
    time::Duration,
};
use thiserror::Error as ThisError;

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

    terminal.clear()?;
    let mut buffer = String::with_capacity(128);

    while !should_quit {
        while event::poll(Duration::ZERO).unwrap() {
            match event::read().unwrap() {
                Event::Key(event) => match event.code {
                    KeyCode::Char(c) => {
                        if c == 'c' && event.modifiers.contains(event::KeyModifiers::CONTROL) {
                            should_quit = true;
                        }

                        buffer.push(c);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(Paragraph::new(buffer.as_str()).black().on_white(), area);
        })?;

        std::thread::sleep(Duration::from_millis(16)); // 60 FPS = 16 millis sleep
    }

    terminal::close()?;

    Ok(())
}
