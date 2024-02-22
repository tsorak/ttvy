use std::sync::Arc;

use crate::chat;
use tokio::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    task::{AbortHandle, JoinHandle},
};

#[derive(Debug)]
pub struct Channel(pub CsSender, pub CsReceiver);
pub type CsSender = Sender<Message>;
type CsReceiver = Receiver<Message>;

#[derive(Debug)]
pub enum WsCommand {
    Message,
    Join,
    Leave,
    Nick,
}

#[derive(Debug)]
pub struct Message(WsCommand, String);

impl Channel {
    pub fn new(bfr: usize) -> Self {
        let (tx, rx) = channel(bfr);
        Self(tx, rx)
    }

    pub fn init(
        (sup_tx, mut sup_rx): (CsSender, CsReceiver),
        connect_options: chat::ConnectOptions,
        chat_config: Arc<Mutex<chat::Config>>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut chat_handle: Option<AbortHandle> = None;
            let mut restart_handle: Option<AbortHandle> = None;
            let mut chat_tx: Option<Sender<String>> = None;

            let connect_options = Arc::new(Mutex::new(connect_options));

            loop {
                match sup_rx.recv().await {
                    None => (),
                    Some(Message(WsCommand::Join, channel_name)) => {
                        if let Some(chat_handle) = &chat_handle {
                            restart_handle.as_ref().unwrap().abort();
                            chat_handle.abort();
                        }

                        {
                            connect_options.lock().await.channel = channel_name.clone();
                        }

                        let (tx, handle) =
                            connect_chat(connect_options.clone(), chat_config.clone());
                        chat_tx = Some(tx);
                        chat_handle = Some(handle.abort_handle());

                        //hacky restart functionality
                        let sup_tx = sup_tx.clone();
                        restart_handle = Some(
                            tokio::spawn(async move {
                                let _ = handle.await;
                                Channel::send(&sup_tx, ("join", &channel_name))
                            })
                            .abort_handle(),
                        );
                    }
                    Some(Message(WsCommand::Leave, _)) => {
                        if let Some(chat_handle) = &chat_handle {
                            restart_handle.as_ref().unwrap().abort();
                            chat_handle.abort();
                            println!("PoroSad ANYWAYS...")
                        } else {
                            println!("You aren't even in a channel LuL!")
                        }
                    }
                    Some(Message(WsCommand::Message, message)) => {
                        if let Some(tx) = &chat_tx {
                            let _ = tx.send(message).await;
                        }
                    }
                    Some(Message(WsCommand::Nick, name)) => {
                        let mut c = connect_options.lock().await;
                        println!("Nick set to: {}", &name);
                        c.nick = Some(name);
                    }
                }
            }
        })
    }

    pub fn send(tx: &Sender<Message>, (cmd, arg1): (&str, &str)) {
        let tx = tx.clone();

        if let Ok(Message(cmd, arg1)) = (cmd, arg1).try_into() {
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
            ("m", message) => Ok(Self(WsCommand::Message, message.to_owned())),
            ("nick", name) => Ok(Self(WsCommand::Nick, name.to_owned())),
            _ => Err(()),
        }
    }
}

fn connect_chat(
    connect_options: Arc<Mutex<chat::ConnectOptions>>,
    chat_config: Arc<Mutex<chat::Config>>,
) -> (Sender<String>, JoinHandle<()>) {
    let (tx, rx) = channel::<String>(10);

    let handle = tokio::spawn(async move {
        let chat_config = chat_config.clone();
        chat::init(connect_options, chat_config, rx).await;
    });

    (tx, handle)
}
