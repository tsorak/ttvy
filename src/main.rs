mod chat;
mod chat_supervisor;
mod config;
mod input;
mod log;

use std::sync::Arc;

use chat_supervisor as cs;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut conf = config::Config::new();
    conf.init();

    let stdin_channel = input::Channel::new(10);
    let cs::Channel(sup_tx, sup_rx) = cs::Channel::new(10);
    let chat_config = Arc::new(Mutex::new(chat::Config::default()));

    let handles = [
        stdin_channel.init(),
        cs::Channel::init(sup_rx, chat_config.clone()),
    ];

    if let Some(ch) = conf.initial_channel {
        cs::Channel::send(&sup_tx, ("join", &ch));
    } else {
        print_help();
    }

    command_loop(chat_config, stdin_channel, sup_tx).await;

    for handle in handles {
        handle.abort();
    }
    std::process::exit(0)
}

async fn command_loop(
    chat_config: Arc<Mutex<chat::Config>>,
    mut stdin_channel: input::Channel,
    sup_tx: cs::CsSender,
) {
    println!("Entering command read loop.");

    loop {
        let stdinput = match stdin_channel.recieve().await {
            Some(s) => s,
            None => continue,
        };

        if let Some((cmd, args)) = input::parse_command(&stdinput) {
            let arg1 = args[0];

            match (cmd, arg1) {
                (_cmd, "") => continue,
                ("join", ch) | ("j", ch) => cs::Channel::send(&sup_tx, ("join", ch)),
                _ => continue,
            }
        } else {
            match stdinput.as_str() {
                "q" => {
                    println!("Bye bye");
                    break;
                }
                "leave" | "ds" | "d" => cs::Channel::send(&sup_tx, ("leave", "")),
                "pad" => {
                    let cfg = &mut chat_config.lock().await;
                    cfg.newline_padding = !cfg.newline_padding;
                    log::chat::pad_status(cfg.newline_padding);
                }
                "color" => {
                    let cfg = &mut chat_config.lock().await;
                    cfg.color_sender = !cfg.color_sender;
                    log::chat::color_status(cfg.color_sender);
                }
                "c" => clear(),
                "h" => print_help(),
                _ => continue,
            }
        }
    }
}

fn print_help() {
    println!(
        "\
        [MAIN]\n\
        join(j) [CHANNEL]: join the specified Twitch chatroom\n\
        leave(d,ds): leave the current chatroom\n\n\
        [CHAT SETTINGS]\n\
        pad: print an empty newline between each message\n\
        color: color usernames\n\n\
        [MISC]\n\
        help(h): print this clump of text\n\
        "
    );
}

fn clear() {
    println!("{esc}c", esc = 27 as char);
}
