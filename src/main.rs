mod input;

#[tokio::main]
async fn main() {
    let mut chat = ttvy_core::chat::Chat::new();
    chat.init().await;

    let mut stdin = input::Channel::new(10);
    let _stdin_handle = stdin.init();

    chat.config.save().await;

    loop {
        tokio::select! {
            msg = chat.receive() => {
                println!("{}: {}", msg.author, msg.message);
            }
            Some(msg) = stdin.receive() => {
                let _ = chat.send(msg).await;
            }
        }
    }
}
