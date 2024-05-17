use std::collections::{HashMap, HashSet};
use std::time::Duration;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use hkb_core::database::init_database;
use hkb_core::database::services::reminders::*;
use hkb_core::logger::{self, debug, error, info, AppenderType};
use hkb_daemon_core::client::{Client, ClientError};
use hkb_daemon_core::server::Server;
use hkb_date::{date::SimpleDate, duration::HumanizedDuration};
use notify_rust::{Notification, Timeout};
use tokio::net::UnixStream;

pub const CORE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../hkb_core/migrations");

async fn process_connection(stream: UnixStream) {
    let mut client = Client::from_stream(stream);
    let mut alternate_interval = tokio::time::interval(std::time::Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = alternate_interval.tick() => {
                match client.flush().await {
                    Err(ClientError::ConnectionClosed(e)) => {
                        debug!(target: "DAEMON", "Client disconnected: {e:?}");
                        break;
                    }
                    _ => {}
                };
            }

            result = client.read_event() => {
                match result {
                    Ok(event) => {
                        debug!(target: "DAEMON", "Received an event: {event:?}");
                    }
                    Err(ClientError::ConnectionClosed(e)) => {
                        debug!(target: "DAEMON", "Client disconnected: {e:?}");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn handle_reminders_notification() {
    let intervals = vec![
        (
            hkb_date::duration::Duration::Minute(0),
            hkb_date::duration::Duration::Minute(1),
        ),
        (
            hkb_date::duration::Duration::Minute(0),
            hkb_date::duration::Duration::Minute(5),
        ),
        (
            hkb_date::duration::Duration::Minute(6),
            hkb_date::duration::Duration::Minute(15),
        ),
        (
            hkb_date::duration::Duration::Minute(16),
            hkb_date::duration::Duration::Minute(30),
        ),
    ];
    let mut already_reminded: HashMap<String, Vec<i64>> = HashMap::new();

    loop {
        debug!(target: "DAEMON", "Checking reminders to notify!");

        for (start, end) in intervals.iter() {
            let start_date = SimpleDate::local().add_duration(start).unwrap();
            let end_date = SimpleDate::local().add_duration(end).unwrap();
            let timeframe_human_string = end_date - SimpleDate::local();
            let reminded = already_reminded
                .entry(end.to_string())
                .or_insert_with(|| Vec::with_capacity(16));
            let options = vec![
                FetchRemindersOption::RemindAtBetween {
                    start_date,
                    end_date,
                },
                FetchRemindersOption::WithoutIds { ids: &reminded },
            ];
            let reminders: Vec<ReminderData> = fetch_reminders(Some(options)).unwrap_or(vec![]);

            debug!(target: "DAEMON", "Found {} reminders to notify!", reminders.len());

            for reminder in reminders {
                debug!(target: "DAEMON", "Reminder at: {} - current time: {}", reminder.remind_at.to_string(), SimpleDate::local().to_string());

                Notification::new()
                    .summary(
                        format!(
                            "You have a reminder in: {}",
                            timeframe_human_string.to_human_string()
                        )
                        .as_str(),
                    )
                    .body(reminder.note.as_str())
                    .auto_icon()
                    .timeout(Timeout::Milliseconds(3000))
                    .show()
                    .unwrap();

                reminded.push(reminder.id);
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    }
}

#[tokio::main]
async fn main() {
    let database_file_path = dirs::data_local_dir().unwrap().join("hkb/db");
    init_database(database_file_path.to_str().unwrap(), vec![CORE_MIGRATIONS]).unwrap();

    logger::init(Some(vec![AppenderType::FILE, AppenderType::STDOUT]));

    let server = Server::bind();

    info!("Listening: {}", server.get_addr().to_str().unwrap());

    tokio::spawn(async move { handle_reminders_notification().await });

    loop {
        match server.accept().await {
            Ok((socket, addr)) => {
                tokio::spawn(async {
                    process_connection(socket).await;
                });
            }
            Err(_) => error!("Failed to accept a connection ;("),
        }
    }
}
