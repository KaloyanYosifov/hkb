use std::collections::HashSet;
use std::time::Duration;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use hkb_core::database::init_database;
use hkb_core::database::services::reminders::*;
use hkb_core::logger::{self, debug, error, info, AppenderType};
use hkb_daemon_core::server::Server;
use hkb_date::{date::SimpleDate, duration::HumanizedDuration};
use notify_rust::{Notification, Timeout};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{unix::SocketAddr, UnixStream},
};

pub const CORE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../hkb_core/migrations");

async fn process_connection(mut socket: UnixStream, addr: SocketAddr) {
    println!("Got a client: {:?} - {:?}", socket, addr);
    socket.write_all(b"hello world").await.unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).await.unwrap();
    println!("{}", response);
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
                    process_connection(socket, addr).await;
                });
            }
            Err(_) => error!("Failed to accept a connection ;("),
        }
    }
}