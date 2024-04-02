use crossterm::event::{self, Event, KeyCode};
use ratatui::{prelude::*, widgets::*};
use std::{io::Error as IOError, time::Duration};
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

fn ui(frame: &mut Frame) {
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

fn main() -> RenderResult {
    let mut terminal = terminal::init()?;
    let mut should_quit = false;

    terminal.clear()?;
    let mut buffer = String::with_capacity(128);

    while !should_quit {
        // 30 FPS = 33 millis. Since poll is blocking we can simulate it as a sleep
        while event::poll(Duration::from_millis(33)).unwrap() {
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

        terminal.draw(ui)?;
    }

    terminal::close()?;

    Ok(())
}
