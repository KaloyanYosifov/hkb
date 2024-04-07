use components::Navigation;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};
use std::fmt::Display;
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

#[derive(Clone, Copy)]
enum AppView {
    Main,
    Reminders,
}

impl AppView {
    fn next(&self) -> Self {
        match self {
            Self::Main => Self::Reminders,
            Self::Reminders => Self::Main,
        }
    }
}

impl Display for AppView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Main => "Main",
            Self::Reminders => "Reminders",
        };

        write!(f, "{}", text)
    }
}

struct AppState {
    view: AppView,
    editing: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            editing: false,
            view: AppView::Main,
        }
    }
}

impl AppState {
    pub fn set_view(&mut self, view: AppView) {
        self.view = view;
    }

    pub fn get_view(&self) -> &AppView {
        &self.view
    }
}

fn main() -> RenderResult {
    let mut terminal = terminal::init()?;
    let mut should_quit = false;
    let mut app_state = AppState::default();
    let mut main_app = apps::MainApp::new();
    let mut reminders_app = apps::RemindersApp::new();
    let mut navigation =
        Navigation::new("HKB".to_string(), vec![AppView::Main, AppView::Reminders]);

    terminal.clear()?;

    while !should_quit {
        while event::poll(Duration::ZERO).unwrap() {
            if let Ok(event) = event::read() {
                match event {
                    Event::Key(event) => match event.code {
                        KeyCode::Char(c) => {
                            should_quit =
                                c == 'c' && event.modifiers.contains(event::KeyModifiers::CONTROL)
                        }
                        _ => {}
                    },
                    _ => {}
                }

                events::push(event);
            }
        }

        terminal.draw(|frame| {
            let base_layout = Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ],
            )
            .split(frame.size());
            navigation.render(frame, &mut app_state, base_layout[0]);
            frame.render_widget(
                Block::new().borders(Borders::TOP).title("Status Bar"),
                base_layout[2],
            );

            let container = Block::default().title("Left");
            let container_area = container.inner(base_layout[1]);
            frame.render_widget(container, base_layout[1]);

            match app_state.view {
                AppView::Main => main_app.render(frame, &mut app_state, container_area),
                AppView::Reminders => reminders_app.render(frame, &mut app_state, container_area),
            };
        })?;

        events::clear();

        // 60 FPS = 16 millis. Since poll is blocking we can simulate it as a sleep
        thread::sleep(Duration::from_millis(16));
    }

    terminal::close()?;

    Ok(())
}
