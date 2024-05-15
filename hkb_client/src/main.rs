use app_state::AppView;
use components::{Component, Navigation};
use crossterm::event::{self, Event, KeyCode};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use hkb_core::database::init_database;
use hkb_core::logger::{debug, error, init as logger_init};
use hkb_daemon_core::client::{Client, ClientError};
use hkb_daemon_core::frame::Event as FrameEvent;
use ratatui::prelude::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders};
use singleton::set_server_msg_sender;
use std::{io::Error as IOError, thread, time::Duration};
use thiserror::Error as ThisError;

mod app_state;
mod apps;
mod components;
mod events;
mod focus;
mod singleton;
mod terminal;
mod utils;

pub const APP_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub const CORE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../hkb_core/migrations");

#[derive(ThisError, Debug)]
pub enum RendererError {
    #[error("Failed to render output!")]
    FailedToRenderToOutput(#[from] IOError),
    #[error("Failed to initialize terminal")]
    FailedToInitializeTerminal(#[from] terminal::TerminalError),
}

type RenderResult = Result<(), RendererError>;

async fn connect_to_server(mut rx: tokio::sync::mpsc::Receiver<FrameEvent>) {
    let mut result = Client::connect().await;

    while result.is_err() {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        result = Client::connect().await;
    }

    let mut client = result.unwrap();
    let mut alternate_interval = tokio::time::interval(std::time::Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = alternate_interval.tick() => {
                match client.flush().await {
                    Err(ClientError::ConnectionClosed(e)) => {
                        debug!(target: "CLIENT", "Server disconnected: {e:?}");
                        break;
                    }
                    _ => {}
                };
            }

            result = rx.recv() => {
                if let Some(event) = result {
                    debug!(target: "CLIENT", "Queued event: {event:?}");
                    client.queue_event(event);
                }
            }

            result = client.read_event() => {
                match result {
                    Ok(event) => {
                        debug!(target: "CLIENT", "Received an event: {event:?}");
                    }
                    Err(ClientError::ConnectionClosed(e)) => {
                        debug!(target: "CLIENT", "Server disconnected: {e:?}");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // if we are running the client and the connection closed out of the blue
    // while we were connected
    // then we try to connected again until
    drop(client);
    spawn_server_connection_thread(rx);
}

fn spawn_server_connection_thread(rx: tokio::sync::mpsc::Receiver<FrameEvent>) {
    tokio::spawn(async move { connect_to_server(rx).await });
}

fn bootstrap() {
    logger_init(None);

    let database_file_path = dirs::data_local_dir().unwrap().join("hkb/db");
    init_database(
        database_file_path.to_str().unwrap(),
        vec![CORE_MIGRATIONS, APP_MIGRATIONS],
    )
    .expect("Failed to initialize database!");

    let (tx, rx) = tokio::sync::mpsc::channel::<FrameEvent>(16);

    set_server_msg_sender(tx);
    spawn_server_connection_thread(rx);
}

#[tokio::main]
async fn main() -> RenderResult {
    bootstrap();

    let mut terminal = terminal::init()?;
    let mut should_quit = false;
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
                        KeyCode::Esc => app_state::set_editing(false),
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
            navigation.render(frame, base_layout[0]);
            frame.render_widget(
                Block::new()
                    .borders(Borders::TOP)
                    .title(if app_state::is_editing() {
                        "Insert mode"
                    } else {
                        "Normal Mode"
                    }),
                base_layout[2],
            );

            match app_state::get_view() {
                AppView::Main => main_app.render(frame, base_layout[1]),
                AppView::Reminders => reminders_app.render(frame, base_layout[1]),
            };
        })?;

        events::clear();

        // 60 FPS = 16 millis. Since poll is blocking we can simulate it as a sleep
        thread::sleep(Duration::from_millis(16));
    }

    terminal::close()?;

    Ok(())
}
