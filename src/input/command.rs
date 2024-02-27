use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub enum CommandType {
    FetchAuth,
    SetAuth,
    Join,
    Leave,
    SetNick,
    Save,
    ShowConfig,
    Reconnect,
    Exit,
    Echo,
    Clear,
}
use self::CommandType as C;

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
        let (cmd, arg) = {
            match line.splitn(2, ' ').collect::<Vec<&str>>()[..] {
                [cmd] => (cmd, None),
                ["echo", s] => ("echo", Some(s.to_string())),
                [cmd, arg] if !arg.trim().is_empty() => (cmd, Some(arg.to_string())),
                _ => ("", None),
            }
        };

        match (cmd, arg) {
            ("auth", None) => Some((C::FetchAuth, empty_arg())),
            ("auth", Some(token)) => Some((C::SetAuth, token)),
            ("join" | "j", Some(ch)) => Some((C::Join, ch.to_lowercase())),
            ("leave" | "d", None) => Some((C::Leave, empty_arg())),
            ("nick", Some(nick)) => Some((C::SetNick, nick.to_lowercase())),
            ("save" | "s", None) => Some((C::Save, empty_arg())),
            ("show", Some(arg)) if arg == "config" => Some((C::ShowConfig, empty_arg())),
            ("reconnect" | "r", None) => Some((C::Reconnect, empty_arg())),
            ("q", None) => Some((C::Exit, empty_arg())),
            ("echo", Some(s)) => Some((C::Echo, s)),
            ("clear" | "c", None) => Some((C::Clear, empty_arg())),
            _ => None,
        }
    }
}

fn empty_arg() -> String {
    String::new()
}
