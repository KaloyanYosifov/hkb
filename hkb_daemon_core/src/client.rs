use hkb_core::logger::info;
use std::path::PathBuf;
use tokio::net::UnixStream;

pub struct Client {
    sock_file: PathBuf,
    stream: UnixStream,
}

impl Client {
    fn init_socket_file() -> PathBuf {
        let data_dir = dirs::data_dir().unwrap().join("hkb");

        data_dir.join("hkb.sock")
    }

    pub async fn connect() -> Self {
        let sock_file = Self::init_socket_file();
        let sock_file_str = sock_file.to_str().unwrap();

        info!(target: "DAEMON CORE CLIENT", "Connecting to {sock_file_str}");

        let stream = UnixStream::connect(&sock_file).await.unwrap();

        info!(target: "DAEMON CORE CLIENT", "Connected to {sock_file_str}");

        Self { sock_file, stream }
    }
}

impl Client {
    pub async fn on_read<F>(&mut self, callback: F)
    where
        F: Fn(&mut UnixStream),
    {
        if let Ok(_) = self.stream.readable().await {
            callback(&mut self.stream);
        }
    }

    pub async fn on_write<F>(&mut self, callback: F)
    where
        F: Fn(&mut UnixStream),
    {
        if let Ok(_) = self.stream.writable().await {
            callback(&mut self.stream);
        }
    }

    pub fn get_addr(&self) -> &PathBuf {
        &self.sock_file
    }
}
