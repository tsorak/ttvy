use std::time::{Duration, Instant};

use tokio::sync::mpsc::{channel, Receiver, Sender};

const ANTI_SPAM_COOLDOWN: Duration = Duration::from_millis(1000);

const UP_ARROW: &str = "\u{1b}[A";

pub struct UserInput {
    pub(super) tx: Sender<String>,
    pub(super) rx: Option<Receiver<String>>,
}

impl UserInput {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, mut bottleneck_rx) = channel::<String>(buffer_size);
        let (bottleneck_tx, rx) = channel::<String>(buffer_size);

        tokio::spawn(async move {
            let mut last_message = String::new();
            let mut ready_at = Instant::now();

            loop {
                let msg = if let Some(msg) = bottleneck_rx.recv().await {
                    let fmt = msg.trim_matches(' ').to_string();
                    prepend_last_message(fmt, &last_message)
                } else {
                    continue;
                };

                match msg {
                    msg if Instant::now() >= ready_at => {
                        let _ = bottleneck_tx
                            .send(take_or_last(msg, &mut last_message))
                            .await;
                        ready_at = Instant::now() + ANTI_SPAM_COOLDOWN;
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

        Self { tx, rx: Some(rx) }
    }
}

fn take_or_last(msg: String, last: &mut String) -> String {
    if msg.is_empty() {
        last.clone()
    } else {
        last.clone_from(&msg);
        msg
    }
}

fn prepend_last_message(s: String, last_msg: &str) -> String {
    match s.strip_prefix(UP_ARROW) {
        Some(addition) => format!("{last_msg} {addition}"),
        None => s,
    }
}
