use std::io;

use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::text::Line;
use tokio::sync::mpsc::Sender;
use ttvy_core::chat::{Chat, ChatEvent, ChatMessage};

mod input;
use input::CommandMessage;

mod output;
use output::StyleConfig;

mod cli_args;

mod ui;
use ui::{AppEvent, AppState, Tui};

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = cli_args::extract();

    let mut chat = Chat::new();
    chat.init().await;

    if cli.authenticate {
        chat.fetch_auth_token().await;
    }

    if let Some(ch) = cli.initial_channel {
        chat.join(&ch);
    }

    let (user_input_tx, mut user_input_rx) = input::user_channel(10);
    let mut style_config = StyleConfig::new();
    let mut state = AppState::new();

    state.push_line(StyleConfig::system("Type !help for help"));

    ui::install_panic_hook();
    let mut tui = Tui::enter()?;
    let mut events = EventStream::new();

    loop {
        tui.terminal.draw(|f| ui::draw(f, &state))?;

        tokio::select! {
            event = chat.receive() => {
                push_event(&mut state, &style_config, event);
            }
            Some(Ok(event)) = events.next() => {
                match ui::handle_event(event, &mut state) {
                    Some(AppEvent::Submit(line)) => {
                        if matches!(
                            route_input(line, &user_input_tx, &mut chat, &mut style_config, &mut state).await,
                            CommandLoopEvent::Exit
                        ) {
                            break;
                        }
                    }
                    Some(AppEvent::Quit) => break,
                    None => {}
                }
            }
            Some(msg) = user_input_rx.recv() => {
                let _ = chat.send(msg).await;
            }
        }
    }

    drop(tui);
    println!("Goodbye");
    Ok(())
}

fn push_event(state: &mut AppState, style: &StyleConfig, event: ChatEvent) {
    match event {
        ChatEvent::Message(msg) => push_chat(state, style, &msg),
        ChatEvent::System(s) => push_system(state, &s),
    }
}

fn push_system(state: &mut AppState, text: &str) {
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            continue;
        }
        state.push_line(StyleConfig::system(line.to_string()));
    }
}

fn push_chat(state: &mut AppState, style: &StyleConfig, msg: &ChatMessage) {
    if style.pad {
        state.push_line(Line::from(""));
    }
    state.push_line(style.display(msg));
}

enum CommandLoopEvent {
    Continue,
    Exit,
}

async fn route_input(
    line: String,
    user_input_tx: &Sender<String>,
    chat: &mut Chat,
    style_config: &mut StyleConfig,
    state: &mut AppState,
) -> CommandLoopEvent {
    if let Some(rest) = line.strip_prefix('!') {
        if let Some(cmd) = CommandMessage::parse(rest) {
            return handle_command(cmd, chat, style_config, state).await;
        }
        state.push_line(StyleConfig::system(format!("Unknown command: !{rest}")));
        return CommandLoopEvent::Continue;
    }

    let _ = user_input_tx.send(line).await;
    CommandLoopEvent::Continue
}

async fn handle_command(
    cmd: CommandMessage,
    chat: &mut Chat,
    style_config: &mut StyleConfig,
    state: &mut AppState,
) -> CommandLoopEvent {
    match cmd {
        CommandMessage::FetchAuth => {
            chat.fetch_auth_token().await;
        }
        CommandMessage::SetAuth(token) => {
            chat.config.oauth.replace(token);
        }
        CommandMessage::SetNick(nick) => {
            chat.config.nick.replace(nick);
        }
        CommandMessage::Join(channel) => chat.join(&channel),
        CommandMessage::Leave => chat.leave().await,
        CommandMessage::Save => chat.save().await,
        CommandMessage::ShowConfig => {
            for line in format!("{:#?}", chat.config).lines() {
                state.push_line(StyleConfig::system(line.to_string()));
            }
        }
        CommandMessage::Reconnect => chat.reconnect().await,
        CommandMessage::Exit => return CommandLoopEvent::Exit,
        CommandMessage::Echo(s) => {
            state.push_line(StyleConfig::system(format!("echo: {s}")));
        }
        CommandMessage::Clear => state.clear_messages(),
        CommandMessage::Help => {
            for line in HELP_TEXT.lines() {
                state.push_line(StyleConfig::system(line.to_string()));
            }
        }
        CommandMessage::Color => style_config.color = !style_config.color,
        CommandMessage::Pad => style_config.pad = !style_config.pad,
    };
    CommandLoopEvent::Continue
}

const HELP_TEXT: &str = "\
[MAIN]
!join(j) [CHANNEL]: Join the specified Twitch chatroom
!leave(d): Leave the current chatroom
!auth: (Re)authenticate with twitch (required in order to send messages)
!auth [TOKEN]: manually set auth token
!nick [NAME]: Set nickname (This needs to be the name of the channel you authenticated as)
!reconnect(r): Reconnect to the last channel

[CHAT SETTINGS]
!color: Color usernames
!pad: Print an empty newline between each message
!debug: Print various junk that Twitch sends

[MISC]
!show config: Prints the current config
!q: Quit the application
!c: Clear the screen
!help(h): Print this clump of text

Editing NICK or AUTH when connected to a chatroom will not take effect, reconnect to apply.
";
