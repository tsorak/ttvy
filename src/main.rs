use input::{CommandMessage, CommandType, Input};
use ttvy_core::chat::Chat;

mod input;

#[tokio::main]
async fn main() {
    let mut chat = Chat::new();
    chat.init().await;

    let mut input = Input::new();
    let (_handle, mut user_input_rx, mut command_rx) = input.init();

    loop {
        tokio::select! {
            msg = chat.receive() => {
                println!("{}: {}", msg.author, msg.message);
            }
            Some(msg) = user_input_rx.recv() => {
                let _ = chat.send(msg).await;
            }
            Some(cmd) = command_rx.recv() => {
                handle_command(cmd, &mut chat).await;
            }
        }
    }
}

async fn handle_command(cmd: CommandMessage, chat: &mut Chat) {
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
    }
}
