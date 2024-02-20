mod chat;
mod chat_supervisor;
mod input;

use chat_supervisor as cs;

#[tokio::main]
async fn main() {
    let mut input = input::Channel::new(10);
    let cs::Channel(sup_tx, sup_rx) = cs::Channel::new(10);

    input.init_stdin_read_loop();
    cs::Channel::init(sup_rx);

    println!("Entering command read loop");
    loop {
        let stdinput = match input.recieve().await {
            Some(s) => s,
            None => continue,
        };

        if let Some((cmd, args)) = input::parse_command(&stdinput) {
            let arg1 = args[0];

            match (cmd, arg1) {
                (_cmd, "") => continue,
                ("join", ch) | ("j", ch) => cs::Channel::send(&sup_tx, ("join", ch)),
                _ => continue,
            }
        } else {
            match stdinput.as_str() {
                "q" => println!("Bye bye"),
                "leave" | "ds" | "d" => cs::Channel::send(&sup_tx, ("leave", "")),
                _ => continue,
            }
        }
    }
}
