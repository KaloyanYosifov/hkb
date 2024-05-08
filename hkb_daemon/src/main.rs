use std::time::Duration;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use hkb_core::database::init_database;
use hkb_core::database::services::reminders::*;
use hkb_core::logger::{self, debug, error, info, AppenderType};
use hkb_daemon_core::server::Server;
use hkb_date::date::SimpleDate;
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

    // spawn thread for reminders
    tokio::spawn(async {
        loop {
            debug!(target: "DAEMON", "Checking reminders to notify!");

            let reminders = fetch_reminders(Some(vec![FetchRemindersOption::RemindAtBetween {
                start_date: SimpleDate::local(),
                end_date: SimpleDate::local()
                    .add_duration(hkb_date::date::Duration::Minute(15))
                    .unwrap(),
            }]))
            .unwrap_or(vec![]);

            debug!(target: "DAEMON", "Found {} reminders to notify!", reminders.len());
            debug!("{:?}", reminders);

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
