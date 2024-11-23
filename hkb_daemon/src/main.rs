use std::collections::HashMap;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use hkb_core::database::init_database;
use hkb_core::database::services::reminders::*;
use hkb_core::logger::{self, debug, error, info, AppenderType};
use hkb_daemon_core::client::{Client, ClientError};
use hkb_daemon_core::server::Server;
use hkb_date::date::SimpleDate;
use notify_rust::{Notification, Timeout};
use tokio::net::UnixStream;

mod audio;

const INTERVALS: [(
    hkb_date::duration::Duration,
    hkb_date::duration::Duration,
    &str,
); 4] = [
    (
        hkb_date::duration::Duration::Minute(0),
        hkb_date::duration::Duration::Minute(1),
        "1 minute",
    ),
    (
        hkb_date::duration::Duration::Minute(0),
        hkb_date::duration::Duration::Minute(5),
        "5 minutes",
    ),
    (
        hkb_date::duration::Duration::Minute(6),
        hkb_date::duration::Duration::Minute(15),
        "15 minutes",
    ),
    (
        hkb_date::duration::Duration::Minute(16),
        hkb_date::duration::Duration::Minute(30),
        "30 minutes",
    ),
];
const CORE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../hkb_core/migrations");

async fn process_connection(stream: UnixStream) {
    let mut client = Client::from_stream(stream);
    let mut alternate_interval = tokio::time::interval(std::time::Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = alternate_interval.tick() => {
                if let Err(ClientError::ConnectionClosed(e)) = client.flush().await {
                    debug!(target: "DAEMON", "Client disconnected: {e:?}");
                    break;
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

async fn handle_reminding(already_reminded: &mut HashMap<String, Vec<i64>>) {
    debug!(target: "DAEMON", "Checking reminders to notify!");

    let mut has_reminded = false;

    for (start, end, humanized_timeframe) in INTERVALS.iter() {
        let start_date = SimpleDate::local().add_duration(start).unwrap();
        let end_date = SimpleDate::local().add_duration(end).unwrap();
        let reminded = already_reminded
            .entry(end.to_string())
            .or_insert_with(|| Vec::with_capacity(16));
        let options = vec![
            ReminderQueryOptions::RemindAtBetween {
                start_date,
                end_date,
            },
            ReminderQueryOptions::WithoutIds { ids: reminded },
        ];
        let reminders: Vec<ReminderData> = fetch_reminders(Some(options)).unwrap_or_default();

        debug!(target: "DAEMON", "Found {} reminders to notify!", reminders.len());

        if !has_reminded {
            has_reminded = !reminders.is_empty();
        }

        for reminder in reminders {
            debug!(target: "DAEMON", "Reminder at: {} - current time: {}", reminder.remind_at.to_string(), SimpleDate::local().to_string());

            Notification::new()
                .summary(format!("You have a reminder in: {}", humanized_timeframe).as_str())
                .body(reminder.note.as_str())
                .auto_icon()
                .timeout(Timeout::Milliseconds(3000))
                .show()
                .unwrap();

            reminded.push(reminder.id);
        }
    }

    if has_reminded {
        // TOOD: play sounds from data directory
        let _ = audio::play_audio("notification.wav".to_string()).await;
    }
}

async fn handle_cleaning_reminders() {
    debug!(target: "DAEMON", "Checking if we should cleanup old reminders.");

    let result = delete_reminders(ReminderQueryOptions::RemindAtLe {
        date: SimpleDate::local()
            .sub_duration(hkb_date::duration::Duration::Day(1))
            .unwrap(),
    });

    match result {
        Ok(_) => {
            debug!(target: "DAEMON", "Successfully deleted old reminders!");
        }
        Err(e) => {
            error!(target: "DAEMON", "Failed to cleanup old reminders! {}", e.to_string());
        }
    }
}

async fn handle_reminders() {
    let mut already_reminded: HashMap<String, Vec<i64>> = HashMap::new();
    let mut cleanup_reminders_interval =
        tokio::time::interval(tokio::time::Duration::from_secs(60 * 5));
    let mut reminder_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = reminder_interval.tick() => {
                handle_reminding(&mut already_reminded).await;
            }
            _ = cleanup_reminders_interval.tick() => {
                handle_cleaning_reminders().await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let database_file_path = dirs::data_local_dir().unwrap().join("hkb/db");
    init_database(database_file_path.to_str().unwrap(), vec![CORE_MIGRATIONS]).unwrap();

    logger::init(Some(vec![AppenderType::FILE, AppenderType::STDOUT]));

    let server = Server::bind();

    info!("Listening: {}", server.get_addr().to_str().unwrap());

    tokio::spawn(async move { audio::init().await });
    tokio::spawn(async move { handle_reminders().await });

    loop {
        match server.accept().await {
            Ok((socket, _)) => {
                tokio::spawn(async {
                    process_connection(socket).await;
                });
            }
            Err(_) => error!("Failed to accept a connection ;("),
        }
    }
}
