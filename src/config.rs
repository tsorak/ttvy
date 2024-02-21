use std::env;

pub struct Config {
    pub initial_channel: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            initial_channel: None,
        }
    }

    pub fn init(&mut self) -> &mut Self {
        let args: Vec<String> = env::args().collect();

        self.initial_channel = args.get(1).cloned();

        self
    }
}
