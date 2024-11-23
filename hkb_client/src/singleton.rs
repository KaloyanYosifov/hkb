use hkb_daemon_core::frame::Event as FrameEvent;

use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};

static GLOBAL_SINGLETON: Mutex<Option<Singleton>> = parking_lot::const_mutex(None);

type ServerMsgSender = tokio::sync::mpsc::Sender<FrameEvent>;

#[derive(Debug)]
#[derive(Default)]
pub struct Singleton {
    server_msg_sender: Option<ServerMsgSender>,
}


impl Singleton {
    fn get_global() -> MappedMutexGuard<'static, Self> {
        MutexGuard::map(GLOBAL_SINGLETON.lock(), |reader| {
            reader.get_or_insert_with(Self::default)
        })
    }
}

pub fn set_server_msg_sender(sender: ServerMsgSender) {
    Singleton::get_global().server_msg_sender = Some(sender);
}

pub fn send_server_msg(event: FrameEvent) {
    let msg_sender = &Singleton::get_global().server_msg_sender;

    if let Some(sender) = msg_sender {
        let handle = tokio::runtime::Handle::current();
        let tx = sender.clone();

        handle.spawn(async move { tx.send(event).await.unwrap_or_default() });
    }
}
