mod chat;
mod chat_supervisor;
mod config;
mod input;
mod log;

use std::sync::Arc;
use tokio::sync::Mutex;

use chat_supervisor as sup;

use crate::config::Config;

#[tokio::main]
async fn main() {
    let conf = Arc::new(Mutex::new(Config::new().await));
    Config::set_initial_channel(&conf).await;

    let stdin_channel = input::Channel::new(10);
    let sup::Channel(sup_tx, sup_rx) = sup::Channel::new(10);

    let chat_config = Arc::new(Mutex::new(chat::Config::default()));

    let (stdin_handle, stdin_close_tx) = stdin_channel.init();
    let sup_handle = sup::Channel::init((sup_tx.clone(), sup_rx), &conf, chat_config.clone());

    if let Some(ch) = &conf.lock().await.channel {
        sup::Channel::send(&sup_tx, ("join", ch));
    } else {
        print_help();
    }

    command_loop(conf, chat_config, stdin_channel, sup_tx).await;

    sup_handle.abort();
    stdin_close_tx.send(()).await.unwrap();
    stdin_handle.abort();
    //stdin stuck reading the final line...
    std::process::exit(0);
}

async fn command_loop(
    conf: Arc<Mutex<config::Config>>,
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

        if let Some((cmd, arg1)) = input::parse_command(&stdinput) {
            match (cmd, arg1) {
                (_cmd, "") => continue,
                ("join", ch) | ("j", ch) => sup::Channel::send(&sup_tx, ("join", ch)),
                ("nick", name) => sup::Channel::send(&sup_tx, ("nick", name)),
                _ => send_message(&sup_tx, &stdinput),
            }
        } else {
            match stdinput.as_str() {
                "auth" => Config::fetch_auth_token(&conf),
                "leave" | "ds" | "d" => sup::Channel::send(&sup_tx, ("leave", "")),
                "r" => reconnect(&sup_tx, &conf).await,
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
                "q" => {
                    println!("Bye bye");
                    break;
                }
                "c" => clear(),
                "h" | "help" => print_help(),
                msg => send_message(&sup_tx, msg),
            }
        }
    }
}

fn print_help() {
    println!(
        "\
        [MAIN]\n\
        join(j) [CHANNEL]: Join the specified Twitch chatroom\n\
        leave(d,ds): Leave the current chatroom\n\
        auth: (Re)authenticate with twitch (required in order to send messages)\n\
        nick [NAME]: Set nickname (This needs to be the name of the channel you authenticated as)\n\
        r: Reconnect to the last channel\n\n\
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

fn send_message(sup_tx: &sup::CsSender, msg: &str) {
    sup::Channel::send(sup_tx, ("m", msg));
}

async fn reconnect(sup_tx: &sup::CsSender, conf: &Arc<Mutex<config::Config>>) {
    let conf = conf.clone();
    let conf = conf.lock().await;
    let ch = conf.channel.clone();
    drop(conf);

    if let Some(ch) = ch {
        sup::Channel::send(sup_tx, ("join", &ch));
    } else {
        println!("No recently joined channel to reconnect to");
    }
}

fn clear() {
    println!("{esc}c", esc = 27 as char);
}
