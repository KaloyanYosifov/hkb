use std::fs;
use std::path::PathBuf;
use tokio::io::Interest;
use tokio::net::UnixStream;

pub struct Client {
    sock_file: PathBuf,
    stream: UnixStream,
}

impl Client {
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

    pub async fn connect() -> Self {
        let sock_file = Self::init_socket_file();
        let stream = UnixStream::connect(&sock_file).await.unwrap();

        Self { sock_file, stream }
    }
}

impl Client {
    pub async fn on_read<F>(&mut self, callback: F)
    where
        F: Fn(&mut UnixStream),
    {
        let result = self
            .stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await;

        if let Ok(ready) = result {
            if ready.is_readable() {
                callback(&mut self.stream);
            }
        }
    }

    pub async fn on_write<F>(&mut self, callback: F)
    where
        F: Fn(&mut UnixStream),
    {
        let result = self
            .stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await;

        if let Ok(ready) = result {
            if ready.is_writable() {
                callback(&mut self.stream);
            }
        }
    }

    pub fn get_addr(&self) -> &PathBuf {
        &self.sock_file
    }
}
