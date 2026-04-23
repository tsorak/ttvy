mod command;
mod user_input;

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::mpsc::{channel, Receiver, Sender},
    task::JoinHandle,
};

pub use self::command::CommandMessage;

pub fn start() -> (JoinHandle<()>, Receiver<String>, Receiver<CommandMessage>) {
    let (user_input_tx, user_input_rx) = user_input::channel_pair(10);
    let (command_tx, command_rx) = channel::<CommandMessage>(10);

    let handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdin()).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            process_line(line, &user_input_tx, &command_tx).await;
        }
    });

    (handle, user_input_rx, command_rx)
}

async fn process_line(
    line: String,
    user_input_tx: &Sender<String>,
    command_tx: &Sender<CommandMessage>,
) {
    let line = line.trim();

    if let Some(rest) = line.strip_prefix('!') {
        if let Some(command) = CommandMessage::parse(rest) {
            let _ = command_tx.send(command).await;
        }
    } else {
        let _ = user_input_tx.send(line.to_string()).await;
    }
}
