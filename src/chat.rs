use std::{collections::HashMap, str::FromStr, sync::Arc};

use colored::{ColoredString, Colorize, CustomColor};
use fast_websocket_client as ws;
use tokio::sync::{mpsc::Receiver, Mutex};

use crate::{config, log};

#[derive(Debug)]
pub struct Config {
    pub newline_padding: bool,
    pub color_sender: bool,
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            newline_padding: false,
            color_sender: true,
            debug: false,
        }
    }
}

impl Config {
    async fn debug_enabled(chat_config: &Arc<Mutex<Config>>) -> bool {
        let c = chat_config.lock().await;
        c.debug
    }

    async fn color_enabled(chat_config: &Arc<Mutex<Config>>) -> bool {
        let c = chat_config.lock().await;
        c.color_sender
    }
}

pub async fn init(
    connect_options: Arc<Mutex<config::Config>>,
    chat_config: Arc<Mutex<Config>>,
    mut input_rx: Receiver<String>,
) {
    let mut state = connect_options.lock().await;
    let ttv_channel = state.channel.clone().unwrap();

    let join = format!("JOIN #{}\n\r", ttv_channel);
    let oauth = format!(
        "PASS oauth:{}",
        state.oauth.get_or_insert_with(|| "blah".to_string())
    );
    let nick = format!(
        "NICK {}\n\r",
        state
            .nick
            .get_or_insert_with(|| "justinfan354678".to_string())
    );

    drop(state);

    let mut conn = ws::connect("ws://irc-ws.chat.twitch.tv:80").await.unwrap();
    conn.set_auto_pong(true);

    conn.send_string(&oauth).await.unwrap();
    conn.send_string(&nick).await.unwrap();
    conn.send_string(&join).await.unwrap();
    conn.send_string("CAP REQ :twitch.tv/tags").await.unwrap();

    let mut read_tags_allowed = false;
    let mut last_sent_message = String::new();
    println!("Joined channel #{}", ttv_channel);
    loop {
        tokio::select! {
            res = conn.receive_frame() => {
                match res {
                    Ok(f) => {
                        let msg = if let Ok(s) = std::str::from_utf8(&f.payload) {
                            s.to_string()
                        } else {
                            f.payload
                                .iter()
                                .map(|v| -> char { (*v).into() })
                                .collect::<String>()
                        };
                        handle_websocket_message(msg, &mut read_tags_allowed, &chat_config).await;
                    }
                    Err(e) => {
                        println!("{}", e);
                        break;
                    }
                }
            }
            msg = input_rx.recv() => {
                if let Some(mut msg) = msg {
                    if msg.is_empty() {
                        msg = last_sent_message.clone();
                    }

                    if last_sent_message == msg {
                        if msg.contains(" \u{E0000}") {
                            msg = msg.strip_suffix(" \u{E0000}").unwrap().to_string();
                        } else {
                            msg.push_str(" \u{E0000}");
                        }
                    }

                    last_sent_message = msg.clone();

                    let fmt = format!("PRIVMSG #{} :{}", ttv_channel, &msg);
                    let _ = conn.send_string(&fmt).await;
                }
            }
        };
    }
}

async fn handle_websocket_message(
    msg: String,
    read_tags_allowed: &mut bool,
    chat_config: &Arc<Mutex<Config>>,
) {
    match msg {
        m if m.contains("ACK :twitch.tv/tags") => {
            *read_tags_allowed = true;
        }
        m if *read_tags_allowed && m.contains("PRIVMSG") => {
            if let Some(user_message) = format_user_message_with_tags(chat_config, &m).await {
                print_user_message(chat_config, user_message).await;
            }
        }
        m if m.contains("PRIVMSG") => {
            if let Some(user_message) = format_user_message(&m) {
                print_user_message(chat_config, user_message).await;
            }
        }
        m if Config::debug_enabled(chat_config).await => {
            log::warn(&m);
        }
        _ => (),
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

async fn format_user_message_with_tags(
    chat_config: &Arc<Mutex<Config>>,
    str: &str,
) -> Option<String> {
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
        if Config::color_enabled(chat_config).await {
            let sender = colorise_sender(sender, &tags);
            Some(format!("{}: {}", sender, msg))
        } else {
            Some(format!("{}: {}", sender.bold(), msg))
        }
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
