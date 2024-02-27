use ttvy_core::chat::Chat;

mod input;
use input::{CommandMessage, CommandType, Input};

mod output;
use output::print_chat_message;

#[tokio::main]
async fn main() {
    let mut chat = Chat::new();
    chat.init().await;

    let mut input = Input::new();
    let (_handle, mut user_input_rx, mut command_rx) = input.init();

    loop {
        tokio::select! {
            msg = chat.receive() => {
                print_chat_message(msg);
            }
            Some(msg) = user_input_rx.recv() => {
                let _ = chat.send(msg).await;
            }
            Some(cmd) = command_rx.recv() => {
                let exit = handle_command(cmd, &mut chat).await;
                if exit {
                    break;
                }
            }
        }
    }
}

async fn handle_command(cmd: CommandMessage, chat: &mut Chat) -> bool {
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
        (CommandType::Exit, _) => return true,
        (CommandType::Echo, s) => {
            dbg!(s);
        }
    };
    false
}
