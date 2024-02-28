mod style;
pub use style::StyleConfig;

use ttvy_core::chat::ChatMessage;

pub fn print_chat_message(msg: ChatMessage, style_config: &StyleConfig) {
    let author = style_config.style_author(&msg.author, msg.color.as_deref());
    style_config.print_chat_message(&author, &msg.message);
}
