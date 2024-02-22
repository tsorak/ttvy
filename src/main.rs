mod chat;
mod chat_supervisor;
mod config;
mod input;
mod log;

use std::sync::Arc;
use tokio::sync::Mutex;

use chat_supervisor as sup;

#[tokio::main]
async fn main() {
    let mut conf = config::Config::new();
    conf.init().await;

    let stdin_channel = input::Channel::new(10);
    let sup::Channel(sup_tx, sup_rx) = sup::Channel::new(10);

    let chat_config = Arc::new(Mutex::new(chat::Config::default()));

    let connect_options = chat::ConnectOptions {
        channel: "".to_string(),
        nick: conf.ttv_nick,
        oauth: conf.ttv_token,
    };

    let handles = [
        stdin_channel.init(),
        sup::Channel::init(
            (sup_tx.clone(), sup_rx),
            connect_options,
            chat_config.clone(),
        ),
    ];

    if let Some(ch) = conf.initial_channel {
        sup::Channel::send(&sup_tx, ("join", &ch));
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
    sup_tx: sup::CsSender,
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
                ("join", ch) | ("j", ch) => sup::Channel::send(&sup_tx, ("join", ch)),
                _ => continue,
            }
        } else {
            match stdinput.as_str() {
                "q" => {
                    println!("Bye bye");
                    break;
                }
                "leave" | "ds" | "d" => sup::Channel::send(&sup_tx, ("leave", "")),
                //chat config
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
                "debug" => {
                    let cfg = &mut chat_config.lock().await;
                    cfg.debug = !cfg.debug;
                    log::chat::debug_status(cfg.debug);
                }
                //misc
                "c" => clear(),
                "h" | "help" => print_help(),
                msg => sup::Channel::send(&sup_tx, ("m", msg)),
            }
        }
    }
}

fn print_help() {
    println!(
        "\
        [MAIN]\n\
        join(j) [CHANNEL]: Join the specified Twitch chatroom\n\
        leave(d,ds): Leave the current chatroom\n\n\
        [CHAT SETTINGS]\n\
        color: Color usernames\n\
        pad: Print an empty newline between each message\n\
        debug: Print various junk that Twitch sends\n\n\
        [MISC]\n\
        q: Quit the application\n\
        c: Clear the screen\n\
        help(h): Print this clump of text\n\
        "
    );
}

fn clear() {
    println!("{esc}c", esc = 27 as char);
}
