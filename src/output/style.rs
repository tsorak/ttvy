use std::str::FromStr;

use colored::{ColoredString, Colorize, CustomColor};
use hex_color::HexColor;
use ttvy_core::chat::ChatMessage;

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

    pub fn display(&self, msg: &ChatMessage) {
        let author = self.style_author(&msg.author, msg.color.as_deref());
        let leading_newline = if self.pad { "\n" } else { "" };
        println!("{leading_newline}{author}: {}", msg.message);
    }

    fn style_author(&self, author: &str, color: Option<&str>) -> ColoredString {
        if !self.color {
            return author.bold();
        }

        let Some(hexcolor) = color else {
            return author.bold();
        };

        match HexColor::from_str(hexcolor) {
            Ok(HexColor { r, g, b, .. }) => author.custom_color(CustomColor { r, g, b }).bold(),
            Err(_) => author.bold(),
        }
    }
}
