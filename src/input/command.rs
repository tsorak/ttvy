use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub enum CommandMessage {
    FetchAuth,
    SetAuth(String),
    Join(String),
    Leave,
    SetNick(String),
    Save,
    ShowConfig,
    Reconnect,
    Exit,
    Echo(String),
    Clear,
    Help,
    Color,
    Pad,
}
use self::CommandMessage as C;

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
        let (cmd, arg) = match line.split_once(' ') {
            None => (line, None),
            Some(("echo", s)) => ("echo", Some(s.to_string())),
            Some((cmd, arg)) if !arg.trim().is_empty() => (cmd, Some(arg.to_string())),
            _ => return None,
        };

        match (cmd, arg) {
            ("auth", None) => Some(C::FetchAuth),
            ("auth", Some(token)) => Some(C::SetAuth(token)),
            ("join" | "j", Some(ch)) => Some(C::Join(ch.to_lowercase())),
            ("leave" | "d", None) => Some(C::Leave),
            ("nick", Some(nick)) => Some(C::SetNick(nick.to_lowercase())),
            ("save" | "s", None) => Some(C::Save),
            ("show", Some(arg)) if arg == "config" => Some(C::ShowConfig),
            ("reconnect" | "r", None) => Some(C::Reconnect),
            ("q", None) => Some(C::Exit),
            ("echo", Some(s)) => Some(C::Echo(s)),
            ("clear" | "c", None) => Some(C::Clear),
            ("help" | "h", None) => Some(C::Help),
            ("color", None) => Some(C::Color),
            ("pad", None) => Some(C::Pad),
            _ => None,
        }
    }
}
