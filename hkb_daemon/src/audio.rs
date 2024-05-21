use hkb_core::logger::debug;
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
    while let Some(path) = rx.recv().await {
        debug!(target: "DAEMON", "Playing audio file: {path}.");

        // TODO: add error handling
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        let file = std::io::BufReader::new(std::fs::File::open(&path).unwrap());
        let source = rodio::Decoder::new(file).unwrap();

        sink.append(source);
        sink.sleep_until_end();

        debug!(target: "DAEMON", "Playing of audio file {path} finished!");
    }
}

pub async fn init() {
    debug!(target: "DAEMON_AUDIO", "Initializing audio.");

    let (tx, rx) = mpsc::channel::<String>(32);

    AudioHandle::set_msg_sender(tx);

    handle(rx).await;
}
