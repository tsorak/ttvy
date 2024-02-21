mod chat;
mod chat_supervisor;
mod input;

use std::sync::Arc;

use chat_supervisor as cs;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mut input = input::Channel::new(10);
    let cs::Channel(sup_tx, sup_rx) = cs::Channel::new(10);
    let chat_config = Arc::new(Mutex::new(chat::Config::default()));

    input.init_stdin_read_loop();
    cs::Channel::init(sup_rx, chat_config.clone());

    println!(
        "\
        Entering command read loop.\n\
        \n\
        Commands:\n\
        join(j) [CHANNEL]: join the specified Twitch chatroom\n\
        leave(d,ds): leave the current chatroom\n\
        pad: print an empty newline between each message\n\
        "
    );
    loop {
        let stdinput = match input.recieve().await {
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
                "q" => println!("Bye bye"),
                "leave" | "ds" | "d" => cs::Channel::send(&sup_tx, ("leave", "")),
                "pad" => {
                    let cfg = &mut chat_config.lock().await;
                    cfg.newline_padding = !cfg.newline_padding;
                }
                "c" => clear(),
                _ => continue,
            }
        }
    }
}

fn clear() {
    println!("{esc}c", esc = 27 as char);
}
