use tokio::sync::mpsc::{channel, Sender};

mod chat;
mod chat_supervisor;
mod input;

#[tokio::main]
async fn main() {
    let mut input = input::Channel::new(10);
    let (supervisor_tx, supervisor_rx) = channel::<chat_supervisor::Message>(10);

    input.init_stdin_read_loop();
    chat_supervisor::init(supervisor_rx);

    let mut is_first_loop = true;

    println!("Entering command read loop");
    loop {
        let stdinput = match input.recieve().await {
            Some(s) => s,
            None => continue,
        };

        if let Some(command) = stdinput.split_once(' ') {
            match command {
                (_cmd, "") => continue,
                ("join", ch) | ("j", ch) => {
                    let _ = &supervisor_tx.send(("join", ch).try_into().unwrap()).await;
                }
                _ => continue,
            }
        } else {
            match stdinput.as_str() {
                "q" => {
                    println!("Bye bye");
                }
                "leave" | "ds" | "d" => {
                    let _ = sup_send(&supervisor_tx, ("leave", "")).await;
                }
                _ if is_first_loop => {
                    sup_send(&supervisor_tx, ("join", "")).await;
                }
                _ => continue,
            }
        }

        if is_first_loop {
            is_first_loop = false;
        }
    }
}

async fn sup_send(tx: &Sender<chat_supervisor::Message>, msg: (&str, &str)) {
    let _ = tx.send(msg.try_into().unwrap()).await;
}
