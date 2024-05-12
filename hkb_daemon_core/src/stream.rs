use hkb_core::logger::debug;
use thiserror::Error as ThisError;
use tokio::net::UnixStream;

use crate::frame::{self, Event, FrameSequence, FRAME_SIZE};

#[derive(ThisError, Debug)]
pub enum StreamError {
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

type StreamResult<T> = Result<T, StreamError>;

fn write(stream: &UnixStream, buf: &[u8]) -> StreamResult<()> {
    match stream.try_write(buf) {
        Ok(_) => {
            debug!(target: "DAEMON CORE STREAM", "Sent event");

            Ok(())
        }
        Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
            Err(StreamError::WritesTemporaryBlocked)
        }
        Err(e) => Err(StreamError::FailedToConnect(e)),
    }
}

pub(crate) async fn read_event(stream: &UnixStream) -> StreamResult<Event> {
    if let Ok(_) = stream.readable().await {
        debug!(target: "DAEMON CORE STREAM", "Can read from socket.");

        let mut buf = [0; FRAME_SIZE];

        match stream.try_read(&mut buf) {
            Ok(0) => Err(StreamError::NotReadyToSendEvent),
            Ok(_) => {
                let event = frame::create_frame_from_bytes(buf).get_event();

                if let Some(event) = event {
                    Ok(event)
                } else {
                    debug!(target: "DAEMON CLIENT", "Received a message that is not an event: {event:?}");

                    Err(StreamError::NotEventMessageReceived)
                }
            }
            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                Err(StreamError::NotReadyToSendEvent)
            }
            Err(e) => Err(StreamError::FailedToConnect(e)),
        }
    } else {
        Err(StreamError::NotReadyToSendEvent)
    }
}

pub(crate) async fn send_event(stream: &UnixStream, event: Event) -> StreamResult<()> {
    if let Ok(_) = stream.writable().await {
        debug!(target: "DAEMON CORE STREAM", "Can write to socket.");

        let frame_sequence: FrameSequence = event.into();

        for frame in frame_sequence {
            // TODO: When we have an error
            // send a discard event to the daemon to discard frame sequence
            write(&stream, frame.convert_to_bytes())?;
        }

        Ok(())
    } else {
        Err(StreamError::NotReadyToSendEvent)
    }
}
