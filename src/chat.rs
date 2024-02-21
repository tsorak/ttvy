use std::sync::Arc;

use fast_websocket_client as ws;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Config {
    pub newline_padding: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            newline_padding: true,
        }
    }
}

pub async fn init(ttv_channel: &str, chat_config: Arc<Mutex<Config>>) {
    let join_string = format!("JOIN #{}", ttv_channel);

    let mut c = ws::connect("ws://irc-ws.chat.twitch.tv:80").await.unwrap();
    c.set_auto_pong(true);

    c.send_string("PASS blah\n\r").await.unwrap();
    c.send_string("NICK justinfan354678\n\r").await.unwrap();
    c.send_string(&join_string).await.unwrap();

    println!("Joined channel #{}", ttv_channel);
    loop {
        match c.receive_frame().await {
            Ok(f) => {
                let msg = f
                    .payload
                    .iter()
                    .map(|b| {
                        let b = *b;
                        let c: char = b.into();
                        c
                    })
                    .collect::<String>();

                if let Some(user_message) = format_user_message(&msg) {
                    let cfg = chat_config.lock().await;
                    if cfg.newline_padding {
                        drop(cfg);
                        println!("{}\r\n", user_message);
                    } else {
                        drop(cfg);
                        println!("{}", user_message);
                    }
                } else {
                    dbg!("[UNKNOWN MESSAGE]", &msg);
                }
            }
            Err(e) => {
                dbg!("Error receiving frame", e);
                break;
            }
        }
    }
}

fn format_user_message(str: &str) -> Option<String> {
    let str = str.split_once("\r\n").unwrap().0;
    if !str.contains("PRIVMSG") {
        return None;
    }

    let sender_nick = if let Some((sender_nick, _)) = str.split_once('!') {
        Some(sender_nick.get(1..).unwrap())
    } else {
        None
    };

    let message = str.splitn(3, ':').last().unwrap();

    if let (Some(sender), msg) = (sender_nick, message) {
        Some(format!("{}: {}", sender, msg))
    } else {
        None
    }
}
