use std::{collections::HashMap, str::FromStr, sync::Arc};

use colored::{ColoredString, Colorize, CustomColor};
use fast_websocket_client as ws;
use tokio::sync::Mutex;

use crate::log;

#[derive(Debug, Default)]
pub struct Config {
    pub newline_padding: bool,
}

pub async fn init(ttv_channel: &str, chat_config: Arc<Mutex<Config>>) {
    let join_string = format!("JOIN #{}", ttv_channel);

    let mut c = ws::connect("ws://irc-ws.chat.twitch.tv:80").await.unwrap();
    c.set_auto_pong(true);

    c.send_string("PASS blah\n\r").await.unwrap();
    c.send_string("NICK justinfan354678\n\r").await.unwrap();
    c.send_string(&join_string).await.unwrap();
    c.send_string("CAP REQ :twitch.tv/tags").await.unwrap();

    let mut read_tags_allowed = false;

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

                match msg {
                    m if m.contains("ACK :twitch.tv/tags") => {
                        read_tags_allowed = true;
                    }
                    m if read_tags_allowed && m.contains("PRIVMSG") => {
                        if let Some(user_message) = format_user_message_with_tags(&m) {
                            print_user_message(&chat_config, user_message).await;
                        }
                    }
                    m if m.contains("PRIVMSG") => {
                        if let Some(user_message) = format_user_message(&m) {
                            print_user_message(&chat_config, user_message).await;
                        }
                    }
                    m => {
                        log::warn(&m);
                    }
                }
            }
            Err(e) => {
                dbg!("Error receiving frame", e);
                break;
            }
        }
    }
}

async fn print_user_message(chat_config: &Arc<Mutex<Config>>, user_message: String) {
    let cfg = chat_config.lock().await;
    if cfg.newline_padding {
        drop(cfg);
        println!("{}\r\n", user_message);
    } else {
        drop(cfg);
        println!("{}", user_message);
    }
}

fn format_user_message(str: &str) -> Option<String> {
    let str = str.split_once("\r\n").unwrap().0;
    if !str.contains("PRIVMSG") {
        return None;
    }

    let sender_nick = if let Some((sender_nick, _)) = str.split_once('!') {
        Some(sender_nick.get(1..).unwrap().bold())
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

fn format_user_message_with_tags(str: &str) -> Option<String> {
    let str = str.split_once("\r\n").unwrap().0;
    if !str.contains("PRIVMSG") {
        return None;
    }

    let (tags, _sender_info, message) = {
        let (tags, tail) = match str.split_once(" :") {
            Some((tags, tail)) => (tags, tail),
            None => return None,
        };

        let (sender_info, message) = match tail.split_once(" :") {
            Some((sender_info, message)) => (sender_info, message),
            None => return None,
        };
        (tags, sender_info, message)
    };

    let tags = parse_tags(tags);

    let sender_nick = tags.get("display-name").copied();

    if let (Some(sender), msg) = (sender_nick, message) {
        let sender = colorise_sender(sender, &tags);

        Some(format!("{}: {}", sender, msg))
    } else {
        None
    }
}

fn parse_tags(tags: &str) -> HashMap<&str, &str> {
    tags.split(';')
        .filter_map(|pair| pair.split_once('='))
        .collect()
}

fn colorise_sender(sender: &str, tags: &HashMap<&str, &str>) -> ColoredString {
    if let Some(hexcolor) = tags.get("color") {
        match hex_color::HexColor::from_str(hexcolor) {
            Ok(hex_color::HexColor { r, g, b, .. }) => {
                sender.custom_color(CustomColor { r, g, b }).bold()
            }
            Err(_) => sender.bold(),
        }
    } else {
        sender.bold()
    }
}
