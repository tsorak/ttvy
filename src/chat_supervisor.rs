use crate::chat;
use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::{AbortHandle, JoinHandle},
};

#[derive(Debug)]
pub struct Channel(pub Sender<Message>, pub Receiver<Message>);

#[derive(Debug)]
pub enum WsCommand {
    Join = 0,
    Leave = 1,
}

#[derive(Debug)]
pub struct Message(WsCommand, String);

impl Channel {
    pub fn new(bfr: usize) -> Self {
        let (tx, rx) = channel(bfr);
        Self(tx, rx)
    }

    pub fn init(mut rx: Receiver<Message>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut chat_handle: Option<AbortHandle> = None;

            loop {
                match rx.recv().await {
                    None => (),
                    Some(Message(WsCommand::Join, channel_name)) => {
                        if let Some(chat_handle) = &chat_handle {
                            chat_handle.abort();
                        }

                        chat_handle = Some(
                            tokio::spawn(async move {
                                chat::init(&channel_name).await;
                            })
                            .abort_handle(),
                        );
                    }
                    Some(Message(WsCommand::Leave, _)) => {
                        if let Some(chat_handle) = &chat_handle {
                            chat_handle.abort();
                            println!("PoroSad ANYWAYS...")
                        } else {
                            println!("You aren't even in a channel LuL!")
                        }
                    }
                }
            }
        })
    }

    pub fn send(tx: &Sender<Message>, (cmd, arg1): (&str, &str)) {
        let tx = tx.clone();

        if let Some(Message(cmd, arg1)) = (cmd, arg1).try_into().ok() {
            tokio::spawn(async move {
                let _ = tx.send(Message(cmd, arg1)).await;
            });
        }
    }
}

impl TryFrom<(&str, &str)> for Message {
    type Error = ();

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        match value {
            ("join", channel_name) => Ok(Self(WsCommand::Join, channel_name.to_owned())),
            ("leave", _) => Ok(Self(WsCommand::Leave, String::new())),
            _ => Err(()),
        }
    }
}
