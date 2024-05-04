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
    let server = Server::bind();

    println!("Listening: {}", server.get_addr().to_str().unwrap());

    loop {
        match server.accept().await {
            Ok((socket, addr)) => {
                tokio::spawn(async {
                    process_connection(socket, addr).await;
                });
            }
            Err(_) => println!("Failed to accept a connection ;("),
        }
    }
}
