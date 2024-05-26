use crossterm::{
    cursor as crossterm_cursor,
    terminal::{self as crossterminal},
    ExecutableCommand,
};
use hkb_core::logger::{error, info};
use ratatui::prelude::{CrosstermBackend, Terminal as TuiTerminal};
use std::io::{self, stdout, Stdout};
use std::panic;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum TerminalError {
    #[error("Failed to initialize the terminal!")]
    FailedToInitializeTerminal(#[from] io::Error),
    #[error("Failed to close the terminal!")]
    FailedToCloseTerminal,
}

pub type Terminal = TuiTerminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Terminal, TerminalError> {
    // Return to cooked mode when app panics
    panic::set_hook(Box::new(|e| {
        close().unwrap_or_default();

        // Print panic info
        #[cfg(debug_assertions)]
        eprintln!("{e}");

        #[cfg(not(debug_assertions))]
        eprintln!("Something went horribly wrong. Check the logs!");

        error!(target: "CLIENT_TERMINAL", "PANIC: {e}");
    }));

    stdout().execute(crossterminal::EnterAlternateScreen)?;
    crossterminal::enable_raw_mode()?;

    let terminal = TuiTerminal::new(CrosstermBackend::new(stdout()))?;

    info!(target: "CLIENT_TERMINAL", "Terminal initialized!");

    Ok(terminal)
}

pub fn set_cursor_steady_bar() {
    stdout()
        .execute(crossterm_cursor::SetCursorStyle::SteadyBar)
        .expect("Should have been able to set cursor!");
}

pub fn set_cursor_to_default() {
    stdout()
        .execute(crossterm_cursor::SetCursorStyle::DefaultUserShape)
        .expect("Should have been able to set cursor!");
}

pub fn size() -> (u16, u16) {
    crossterminal::size().unwrap()
}

pub fn close() -> Result<(), TerminalError> {
    {
        if crossterminal::disable_raw_mode().is_err() {
            error!(target: "CLIENT_TERMINAL", "Failed to disable raw mode :/");

            return Err(TerminalError::FailedToCloseTerminal);
        }

        if stdout()
            .execute(crossterminal::LeaveAlternateScreen)
            .is_err()
        {
            error!(target: "CLIENT_TERMINAL", "Failed to leave alternate screen :/");

            return Err(TerminalError::FailedToCloseTerminal);
        }
    }

    info!(target: "CLIENT_TERMINAL", "Terminal closed!");

    Ok(())
}
