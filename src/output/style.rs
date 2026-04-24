use std::str::FromStr;

use hex_color::HexColor;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ttvy_core::chat::ChatMessage;

pub struct StyleConfig {
    pub color: bool,
    pub pad: bool,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            color: true,
            pad: false,
        }
    }
}

impl StyleConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display(&self, msg: &ChatMessage) -> Line<'static> {
        let author_style = self.author_style(msg.color.as_deref());
        Line::from(vec![
            Span::styled(msg.author.clone(), author_style),
            Span::raw(": "),
            Span::raw(msg.message.clone()),
        ])
    }

    pub fn system(text: impl Into<String>) -> Line<'static> {
        Line::from(Span::styled(
            text.into(),
            Style::default().add_modifier(Modifier::DIM),
        ))
    }

    fn author_style(&self, color: Option<&str>) -> Style {
        let base = Style::default().add_modifier(Modifier::BOLD);
        if !self.color {
            return base;
        }
        let Some(hexcolor) = color else {
            return base;
        };
        match HexColor::from_str(hexcolor) {
            Ok(HexColor { r, g, b, .. }) => base.fg(Color::Rgb(r, g, b)),
            Err(_) => base,
        }
    }
}
