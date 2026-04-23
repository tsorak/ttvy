use clap::{
    builder::{
        styling::{AnsiColor, Effects},
        Styles,
    },
    CommandFactory, FromArgMatches, Parser,
};

#[derive(Parser)]
#[command(name = "ttvy", about = "Command-line Twitch chat client", author, version)]
pub struct CliArgs {
    #[arg(help = "Provide a channel to connect to initially")]
    pub initial_channel: Option<String>,

    #[arg(short, long, help = "Sets a new Twitch session token")]
    pub authenticate: bool,
}

pub fn extract() -> CliArgs {
    let matches = CliArgs::command().styles(custom_style()).get_matches();
    CliArgs::from_arg_matches(&matches).unwrap()
}

fn custom_style() -> Styles {
    Styles::styled()
        .header(AnsiColor::White.on_default() | Effects::BOLD)
        .usage(AnsiColor::White.on_default() | Effects::BOLD)
        .literal(AnsiColor::Magenta.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
}
