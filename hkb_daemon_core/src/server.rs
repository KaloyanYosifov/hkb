use std::fs;
use std::path::PathBuf;
use tokio::net::{unix::SocketAddr, UnixListener, UnixStream};

pub struct Server {
    sock_file: PathBuf,
    listener: UnixListener,
}

impl Server {
    fn init_socket_file() -> PathBuf {
        let data_dir = dirs::data_dir().unwrap().join("hkb");
        let sock_file = data_dir.join("hkb.sock");

        if !data_dir.exists() {
            fs::create_dir(&data_dir).unwrap();
        }

        if sock_file.exists() {
            fs::remove_file(&sock_file).unwrap();
        }

        sock_file
    }

    pub fn bind() -> Self {
        let sock_file = Self::init_socket_file();
        let listener = UnixListener::bind(&sock_file).unwrap();

        Self {
            sock_file,
            listener,
        }
    }
}

impl Server {
    pub async fn accept(&self) -> tokio::io::Result<(UnixStream, SocketAddr)> {
        self.listener.accept().await
    }

    pub fn get_addr(&self) -> &PathBuf {
        &self.sock_file
    }
}
