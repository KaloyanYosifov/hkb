use std::time::Duration;

use hkb_core::logger::{self, debug, error, info, AppenderType};
use hkb_daemon_core::server::Server;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{unix::SocketAddr, UnixStream},
};

async fn process_connection(mut socket: UnixStream, addr: SocketAddr) {
    println!("Got a client: {:?} - {:?}", socket, addr);
    socket.write_all(b"hello world").await.unwrap();
    let mut response = String::new();
    socket.read_to_string(&mut response).await.unwrap();
    println!("{}", response);
}

#[tokio::main]
async fn main() {
    logger::init(Some(vec![AppenderType::FILE, AppenderType::STDOUT]));

    let server = Server::bind();

    info!("Listening: {}", server.get_addr().to_str().unwrap());

    // spawn thread for reminders
    tokio::spawn(async {
        loop {
            debug!(target: "DAEMON", "Checking reminders to notify!");

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
