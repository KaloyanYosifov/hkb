use std::collections::HashSet;
use std::time::Duration;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use hkb_core::database::init_database;
use hkb_core::database::services::reminders::*;
use hkb_core::logger::{self, debug, error, info, AppenderType};
use hkb_daemon_core::server::Server;
use hkb_date::{date::SimpleDate, duration::HumanizedDuration};
use notify_rust::{Notification, Timeout};
use tokio::io;
use tokio::net::UnixStream;

pub const CORE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../hkb_core/migrations");

struct Client {
    stream: UnixStream,
}

impl Client {
    pub fn new(stream: UnixStream) -> Self {
        Self { stream }
    }
}

impl Client {
    pub async fn handle(&self) {
        loop {
            match self.stream.try_write(b"PING") {
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // do nothing if we have would block
                }
                Err(e) => {
                    debug!(target: "DAEMON", "Client disconnected! {e:?}");
                    break;
                }
                _ => {}
            }

            let mut buf = [0; 4096];
            match self.stream.try_read(&mut buf) {
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // do nothing if we have would block
                }
                Err(_) => {
                    break;
                }
                Ok(_) => {
                    info!("Message from client: {buf:?}");
                }
            }
        }
    }
}

async fn process_connection(stream: UnixStream) {
    println!("Got a client!");
    let client = Client::new(stream);
    client.handle().await;
}

#[tokio::main]
async fn main() {
    let database_file_path = dirs::data_local_dir().unwrap().join("hkb/db");
    init_database(database_file_path.to_str().unwrap(), vec![CORE_MIGRATIONS]).unwrap();

    logger::init(Some(vec![AppenderType::FILE, AppenderType::STDOUT]));

    let server = Server::bind();

    info!("Listening: {}", server.get_addr().to_str().unwrap());

    let mut already_reminded: HashSet<i64> = HashSet::new();
    // spawn thread for reminders
    tokio::spawn(async move {
        loop {
            debug!(target: "DAEMON", "Checking reminders to notify!");

            let mut reminders =
                fetch_reminders(Some(vec![FetchRemindersOption::RemindAtBetween {
                    start_date: SimpleDate::local(),
                    end_date: SimpleDate::local()
                        .add_duration(hkb_date::duration::Duration::Minute(15))
                        .unwrap(),
                }]))
                .unwrap_or(vec![]);

            // TODO: support filtering out ids in reminders service
            reminders = reminders
                .into_iter()
                .filter(|reminder| !already_reminded.contains(&reminder.id))
                .collect();

            debug!(target: "DAEMON", "Found {} reminders to notify!", reminders.len());

            for reminder in reminders {
                debug!(target: "DAEMON", "Reminder at: {} - current time: {}", reminder.remind_at.to_string(), SimpleDate::local().to_string());

                Notification::new()
                    .summary(
                        format!(
                            "You have a reminder in: {}",
                            (reminder.remind_at - SimpleDate::local()).to_human_string()
                        )
                        .as_str(),
                    )
                    .body(reminder.note.as_str())
                    .auto_icon()
                    .timeout(Timeout::Milliseconds(3000))
                    .show()
                    .unwrap();
                already_reminded.insert(reminder.id);
            }

            std::thread::sleep(Duration::from_secs(5));
        }
    });

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
