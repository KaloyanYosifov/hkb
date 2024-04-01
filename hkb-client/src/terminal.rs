use crossterm::{
    terminal::{self as crossterminal},
    ExecutableCommand,
};
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
        eprintln!("{e}");
    }));

    stdout().execute(crossterminal::EnterAlternateScreen)?;
    crossterminal::enable_raw_mode()?;

    Ok(TuiTerminal::new(CrosstermBackend::new(stdout()))?)
}

pub fn close() -> Result<(), TerminalError> {
    {
        if crossterminal::disable_raw_mode().is_err() {
            return Err(TerminalError::FailedToCloseTerminal);
        }

        if stdout()
            .execute(crossterminal::LeaveAlternateScreen)
            .is_err()
        {
            return Err(TerminalError::FailedToCloseTerminal);
        }
    }

    Ok(())
}
