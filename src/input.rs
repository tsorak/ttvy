mod command;
mod user_input;

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub use self::command::{CommandMessage, CommandType};
use self::{command::Command, user_input::UserInput};

pub struct Input {
    command: Command,
    user_input: UserInput,
}

impl Input {
    pub fn new() -> Self {
        let mut user_input = UserInput::new(10);
        user_input.init();

        Self {
            command: Command::new(),
            user_input,
        }
    }

    pub fn init(&mut self) -> (JoinHandle<()>, Receiver<String>, Receiver<CommandMessage>) {
        let user_input_rx = self.user_input.rx.take().expect("Only call init once");
        let command_rx = self.command.rx.take().expect("Only call init once");

        let user_input_tx = self.user_input.tx.clone();
        let command_tx = self.command.tx.clone();

        let h = tokio::spawn(async move {
            let stdin = stdin();
            let mut stdin = BufReader::new(stdin).lines();
            loop {
                if let Ok(Some(line)) = stdin.next_line().await {
                    Self::process_line(line, &user_input_tx, &command_tx).await;
                }
            }
        });

        (h, user_input_rx, command_rx)
    }

    async fn process_line(
        line: String,
        user_input_tx: &Sender<String>,
        command_tx: &Sender<CommandMessage>,
    ) {
        let line = line.trim();

        if line.starts_with('!') {
            let line = line.strip_prefix('!').unwrap();
            if let Some(command) = Command::parse(line) {
                let _ = command_tx.send(command).await;
            }
        } else {
            let _ = user_input_tx.send(line.to_string()).await;
        }
    }
}
