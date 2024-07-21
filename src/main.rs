use ttvy_core::chat::Chat;

mod input;
use input::{CommandMessage, CommandType, Input};

mod output;
use output::{print_chat_message, StyleConfig};

mod cli_args;

#[tokio::main]
async fn main() {
    let cli = cli_args::extract();

    let mut chat = Chat::new();
    chat.init().await;

    if cli.authenticate {
        chat.fetch_auth_token().await;
    }

    if let Some(ch) = cli.initial_channel {
        chat.join(&ch);
    }

    let mut input = Input::new();
    let (_handle, mut user_input_rx, mut command_rx) = input.init();

    let mut style_config = StyleConfig::new();

    println!("Type !help for help");
    loop {
        tokio::select! {
            msg = chat.receive() => {
                print_chat_message(msg, &style_config);
            }
            Some(msg) = user_input_rx.recv() => {
                let _ = chat.send(msg).await;
            }
            Some(cmd) = command_rx.recv() => {
                match handle_command(cmd, &mut chat, &mut style_config).await {
                    CommandLoopEvent::Exit => break,
                    CommandLoopEvent::Continue => continue,
                }
            }
        }
    }

    println!("Goodbye");
    //stdin receiver freezes, todo!
    std::process::exit(0);
}

enum CommandLoopEvent {
    Continue,
    Exit,
}

async fn handle_command(
    cmd: CommandMessage,
    chat: &mut Chat,
    style_config: &mut StyleConfig,
) -> CommandLoopEvent {
    match cmd {
        (CommandType::FetchAuth, _) => {
            chat.fetch_auth_token().await;
        }
        (CommandType::SetAuth, token) => {
            chat.config.oauth.replace(token);
        }
        (CommandType::SetNick, nick) => {
            chat.config.nick.replace(nick);
        }
        (CommandType::Join, channel) => chat.join(&channel),
        (CommandType::Leave, _) => chat.leave().await,
        (CommandType::Save, _) => chat.config.save().await,
        (CommandType::ShowConfig, _) => println!("{:#?}", chat.config),
        (CommandType::Reconnect, _) => chat.reconnect(),
        (CommandType::Exit, _) => return CommandLoopEvent::Exit,
        (CommandType::Echo, s) => {
            dbg!(s);
        }
        (CommandType::Clear, _) => clear(),
        (CommandType::Help, _) => print_help(),
        (CommandType::Color, _) => style_config.color = !style_config.color,
        (CommandType::Pad, _) => style_config.pad = !style_config.pad,
    };
    CommandLoopEvent::Continue
}

fn clear() {
    println!("\x1B[2J\x1B[1;1H");
}

fn print_help() {
    println!(
        "\
        [MAIN]\n\
        !join(j) [CHANNEL]: Join the specified Twitch chatroom\n\
        !leave(d): Leave the current chatroom\n\
        !auth: (Re)authenticate with twitch (required in order to send messages)\n\
        !auth [TOKEN]: manually set auth token\n\
        !nick [NAME]: Set nickname (This needs to be the name of the channel you authenticated as)\n\
        !reconnect(r): Reconnect to the last channel\n\n\
        [CHAT SETTINGS]\n\
        !color: Color usernames\n\
        !pad: Print an empty newline between each message\n\
        !debug: Print various junk that Twitch sends\n\n\
        [MISC]\n\
        !show config: Prints the current config\n\
        !q: Quit the application\n\
        !c: Clear the screen\n\
        !help(h): Print this clump of text\n\n\
        Editing NICK or AUTH when connected to a chatroom will not take effect, reconnect to apply.
        "
    );
}
