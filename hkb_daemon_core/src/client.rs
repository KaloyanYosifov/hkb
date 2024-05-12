use hkb_core::logger::{debug, info};
use std::path::PathBuf;
use thiserror::Error as ThisError;
use tokio::net::UnixStream;

use crate::frame::{self, Event, FrameSequence, FRAME_SIZE};

#[derive(ThisError, Debug)]
pub enum ClientError {
    #[error("Writes are temporarily blocked.")]
    WritesTemporaryBlocked,

    #[error("Not ready to send event")]
    NotReadyToSendEvent,

    #[error("Failed to send event to the daemon server.")]
    FailedToConnect(std::io::Error),

    #[error("Reads are temporarily blocked.")]
    ReadsTemporaryBlocked,

    #[error("Not ready to receive event")]
    NotReadyToReceiveEvent,

    #[error("Receive a non event based message from socket")]
    NotEventMessageReceived,
}

type ClientResult<T> = Result<T, ClientError>;

pub struct Client {
    sock_file: PathBuf,
    stream: UnixStream,
    event_queue: Vec<Event>,
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

        Self::from_stream(stream)
    }

    pub fn from_stream(stream: UnixStream) -> Self {
        let mut sock_file = Self::init_socket_file();

        if let Ok(addr) = stream.peer_addr() {
            if let Some(path) = addr.as_pathname() {
                sock_file = path.to_path_buf();
            }
        }

        Self {
            stream,
            sock_file,
            event_queue: Vec::with_capacity(32),
        }
    }
}

impl Client {
    fn write(&self, buf: &[u8]) -> ClientResult<()> {
        match self.stream.try_write(buf) {
            Ok(_) => {
                debug!(target: "DAEMON CORE CLIENT", "Sent event");

                Ok(())
            }
            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                Err(ClientError::WritesTemporaryBlocked)
            }
            Err(e) => Err(ClientError::FailedToConnect(e)),
        }
    }

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

    pub async fn read_event(&self) -> ClientResult<Event> {
        if let Ok(_) = self.stream.readable().await {
            debug!(target: "DAEMON CORE CLIENT", "Can read from socket.");

            let mut buf = [0; FRAME_SIZE];

            match self.stream.try_read(&mut buf) {
                Ok(0) => Err(ClientError::NotReadyToSendEvent),
                Ok(_) => {
                    let event = frame::create_frame_from_bytes(buf).get_event();

                    if let Some(event) = event {
                        Ok(event)
                    } else {
                        debug!(target: "DAEMON CORE CLIENT", "Received a message that is not an event: {event:?}");

                        Err(ClientError::NotEventMessageReceived)
                    }
                }
                Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                    Err(ClientError::NotReadyToSendEvent)
                }
                Err(e) => Err(ClientError::FailedToConnect(e)),
            }
        } else {
            Err(ClientError::NotReadyToSendEvent)
        }
    }

    pub async fn send_event(&self, event: impl AsRef<Event>) -> ClientResult<()> {
        if let Ok(_) = self.stream.writable().await {
            debug!(target: "DAEMON CORE CLIENT", "Can write to socket.");

            let frame_sequence: FrameSequence = event.as_ref().into();

            for frame in frame_sequence {
                // TODO: When we have an error
                // send a discard event to the daemon to discard frame sequence
                self.write(frame.convert_to_bytes())?;
            }

            Ok(())
        } else {
            Err(ClientError::NotReadyToSendEvent)
        }
    }

    pub fn queue_event(&mut self, event: Event) {
        self.event_queue.push(event);
    }

    pub async fn flush(&mut self) -> ClientResult<()> {
        if let Ok(_) = self.stream.writable().await {
            for event in self.event_queue.iter() {
                let frame_sequence: FrameSequence = event.into();

                for frame in frame_sequence {
                    // TODO: When we have an error
                    // send a discard event to the daemon to discard frame sequence
                    self.write(frame.convert_to_bytes())?;
                }
            }

            self.event_queue.clear();

            Ok(())
        } else {
            Err(ClientError::NotReadyToSendEvent)
        }
    }

    pub fn get_addr(&self) -> &PathBuf {
        &self.sock_file
    }
}
