use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub enum CommandType {
    FetchAuth,
    SetAuth,
    SetChannel,
    Leave,
    SetNick,
    Save,
    ShowConfig,
    Reconnect,
}

pub type CommandMessage = (CommandType, String);

pub struct Command {
    pub(super) tx: Sender<CommandMessage>,
    pub(super) rx: Option<Receiver<CommandMessage>>,
}

impl Command {
    pub fn new() -> Self {
        let (tx, rx) = channel::<CommandMessage>(10);
        Self { tx, rx: Some(rx) }
    }

    pub fn parse(line: &str) -> Option<CommandMessage> {
        match line.splitn(2, ' ').collect::<Vec<&str>>()[..] {
            ["auth"] => Some((CommandType::FetchAuth, String::new())),
            ["auth", token] => Some((CommandType::SetAuth, token.to_string())),
            ["join", ch] | ["j", ch] if !ch.is_empty() => {
                Some((CommandType::SetChannel, ch.to_lowercase()))
            }
            ["leave"] | ["d"] => Some((CommandType::Leave, String::new())),
            ["nick", name] if !name.is_empty() => Some((CommandType::SetNick, name.to_lowercase())),
            ["save"] => Some((CommandType::Save, String::new())),
            ["show", "config"] => Some((CommandType::ShowConfig, String::new())),
            ["reconnect"] | ["r"] => Some((CommandType::Reconnect, String::new())),
            _ => None,
        }
    }
}
