use clap::{
    builder::{
        styling::{AnsiColor, Effects},
        Styles,
    },
    crate_authors, crate_version, Args, Command,
};

pub fn extract() -> ProvidedArgs {
    let cli = Command::new("ttvy")
        .about("Command-line Twitch chat client")
        .author(crate_authors!())
        .version(crate_version!())
        .styles(custom_style());

    ProvidedArgs::from_cli(cli)
}

#[derive(Args)]
struct CliArgs {
    #[arg(help = "Provide a channel to connect to initially")]
    initial_channel: Option<String>,

    #[arg(short, long, help = "Sets a new Twitch session token")]
    authenticate: bool,
}

pub struct ProvidedArgs {
    pub initial_channel: Option<String>,
    pub authenticate: bool,
}

impl ProvidedArgs {
    pub fn from_cli(cli: Command) -> Self {
        let m = CliArgs::augment_args(cli).get_matches();

        Self {
            initial_channel: m.get_one::<String>("initial_channel").cloned(),
            authenticate: m.get_flag("authenticate"),
        }
    }
}

fn custom_style() -> Styles {
    Styles::styled()
        .header(AnsiColor::White.on_default() | Effects::BOLD)
        .usage(AnsiColor::White.on_default() | Effects::BOLD)
        .literal(AnsiColor::Magenta.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
}
