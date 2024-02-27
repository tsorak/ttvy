use std::time::Duration;

use tokio::{
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

const UP_ARROW: &str = "\u{1b}[A";

pub struct UserInput {
    pub(super) tx: Sender<String>,
    bottleneck_rx: Option<Receiver<String>>,
    bottleneck_tx: Option<Sender<String>>,
    pub(super) rx: Option<Receiver<String>>,
}

impl UserInput {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, bottleneck_rx) = channel::<String>(buffer_size);
        let (bottleneck_tx, rx) = channel::<String>(buffer_size);

        Self {
            tx,
            bottleneck_rx: Some(bottleneck_rx),
            bottleneck_tx: Some(bottleneck_tx),
            rx: Some(rx),
        }
    }

    pub fn init(&mut self) -> Option<JoinHandle<()>> {
        if self.bottleneck_rx.is_none() || self.bottleneck_tx.is_none() {
            return None;
        }

        let mut rx = self.bottleneck_rx.take().unwrap();
        let tx = self.bottleneck_tx.take().unwrap();

        let handle = tokio::spawn(async move {
            let mut last_message = String::new();
            let mut timeout = anti_spam_timeout(0);

            loop {
                let msg = if let Some(msg) = rx.recv().await {
                    let fmt = msg.trim_matches(' ').to_string();
                    prepend_last_message(fmt, &last_message)
                } else {
                    continue;
                };

                match msg {
                    msg if timeout.is_finished() => {
                        let _ = tx.send(msg.if_empty_do(&mut last_message)).await;
                        timeout = anti_spam_timeout(1000);
                    }
                    msg if !msg.is_empty() => {
                        //Incoming message before timeout has passed.
                        //
                        //Set this msg as last_message to prevent it from dissapearing
                        //If the user decides to spam enter after this, the previous match arm
                        //will catch it (once timeout passes) and pick out the msg received in this arm.
                        last_message = msg;
                    }
                    _ => (),
                }
            }
        });

        Some(handle)
    }
}

fn anti_spam_timeout(ms: u64) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    })
}

trait IfEmptyDo {
    fn if_empty_do(&self, fallback: &mut String) -> String;
}

impl IfEmptyDo for String {
    fn if_empty_do(&self, fallback: &mut String) -> String {
        if self.is_empty() {
            fallback.clone()
        } else {
            *fallback = self.clone();
            self.clone()
        }
    }
}

fn prepend_last_message(s: String, last_msg: &String) -> String {
    if !s.starts_with(UP_ARROW) {
        return s;
    }

    match s.splitn(2, UP_ARROW).collect::<Vec<_>>()[..] {
        ["", addition] => format!("{last_msg} {addition}"),
        _ => s,
    }
}
