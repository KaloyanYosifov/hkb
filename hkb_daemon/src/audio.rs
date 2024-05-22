use std::borrow::Borrow;

use hkb_core::logger::{debug, error};
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use thiserror::Error as ThisError;
use tokio::sync::mpsc;

#[derive(ThisError, Debug)]
pub enum AudioError {
    #[error("No msg channel found!")]
    FailedToFindMsgChannel,

    #[error("Failed to play audio!")]
    FailedToPlayAudio,
}

static GLOBAL_AUDIO_HANDLE: Mutex<Option<AudioHandle>> = parking_lot::const_mutex(None);

type AudioMessageSender = mpsc::Sender<String>;

struct AudioHandle {
    audio_msg_sender: Option<AudioMessageSender>,
}

impl Default for AudioHandle {
    fn default() -> Self {
        Self {
            audio_msg_sender: None,
        }
    }
}

impl AudioHandle {
    fn get_global() -> MappedMutexGuard<'static, Self> {
        MutexGuard::map(GLOBAL_AUDIO_HANDLE.lock(), |reader| {
            reader.get_or_insert_with(Self::default)
        })
    }

    fn set_msg_sender(sender: AudioMessageSender) {
        Self::get_global().audio_msg_sender = Some(sender);
    }

    async fn play_audio(path: String) -> Result<(), AudioError> {
        if let Some(sender) = &Self::get_global().audio_msg_sender {
            debug!(target: "DAEMON_AUDIO", "Sending msg to play audio {path}");

            let tx = sender.clone();

            if let Ok(_) = tx.send(path).await {
                debug!(target: "DAEMON_AUDIO", "MSG sent");

                Ok(())
            } else {
                debug!(target: "DAEMON_AUDIO", "Failed to send MSG");
                Err(AudioError::FailedToPlayAudio)
            }
        } else {
            Err(AudioError::FailedToFindMsgChannel)
        }
    }
}

pub async fn play_audio(path: String) -> Result<(), AudioError> {
    AudioHandle::play_audio(path).await
}

pub async fn handle(mut rx: mpsc::Receiver<String>) {
    let sound_directory = dirs::data_local_dir().unwrap().join("hkb/sound");

    while let Some(path) = rx.recv().await {
        let mut file_path = std::path::PathBuf::from(&path);

        if file_path.is_relative() {
            file_path = sound_directory.join(file_path);
        }

        let path = file_path.to_str().unwrap();

        if !file_path.exists() {
            error!(target: "DAEMON_AUDIO", "Audio file does not exist: {}", file_path.to_str().unwrap());

            continue;
        }

        debug!(target: "DAEMON_AUDIO", "Playing audio file: {}.", path);

        // TODO: add error handling
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        let file = std::io::BufReader::new(std::fs::File::open(&file_path).unwrap());
        let source = rodio::Decoder::new(file).unwrap();

        sink.append(source);
        sink.sleep_until_end();

        debug!(target: "DAEMON_AUDIO", "Playing of audio file {} finished!", path);
    }
}

pub async fn init() {
    debug!(target: "DAEMON_AUDIO", "Initializing audio.");

    let (tx, rx) = mpsc::channel::<String>(32);

    AudioHandle::set_msg_sender(tx);

    handle(rx).await;
}
