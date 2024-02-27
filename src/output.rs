use std::str::FromStr;

use colored::{ColoredString, Colorize, CustomColor};
use hex_color::HexColor::{self};

use ttvy_core::chat::ChatMessage;

pub fn print_chat_message(msg: ChatMessage) {
    let author = color_author(msg.author.as_str(), msg.color.as_deref());
    println!("{}: {}", author, msg.message);
}

fn color_author(author: &str, color: Option<&str>) -> ColoredString {
    if let Some(hexcolor) = color {
        match HexColor::from_str(hexcolor) {
            Ok(hex_color::HexColor { r, g, b, .. }) => {
                author.custom_color(CustomColor { r, g, b }).bold()
            }
            Err(_) => author.bold(),
        }
    } else {
        author.bold()
    }
}
