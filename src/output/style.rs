use std::str::FromStr;

use colored::{ColoredString, Colorize, CustomColor};
use hex_color::HexColor::{self};

pub struct StyleConfig {
    pub color: bool,
    pub pad: bool,
    pub debug: bool,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            color: true,
            pad: false,
            debug: false,
        }
    }
}

impl StyleConfig {
    pub fn new() -> Self {
        Self::default()
    }

    // pub async fn update_inner(config: &Arc<Mutex<Self>>, accessor: Box<dyn FnOnce(&mut Self)>) {
    //     let config = config.clone();
    //     let lock = config.lock().await;
    // }

    pub fn style_author(&self, author: &str, color: Option<&str>) -> ColoredString {
        if self.color {
            color_author(author, color)
        } else {
            author.bold()
        }
    }

    pub fn print_chat_message(&self, author: &ColoredString, message: &str) {
        if self.pad {
            println!("\n{}: {}", author, message);
        } else {
            println!("{}: {}", author, message);
        }
    }
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
