use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent},
    style::Print,
    terminal::{Clear, ClearType},
};
use std::{collections::HashMap, io::stdout};

mod reader;

use reader::Reader;

type CmdBuf = Vec<char>;
const NUMBER_KEYS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const COMMAND_KEYS: [char; 1] = ['d'];
const MOTION_KEYS: [char; 4] = ['h', 'j', 'k', 'l'];

struct VimState {
    command: String,
    count: String,
    motion: String,
    // expected_key: ExpectedKey,
    motion_handlers: HashMap<char, Box<dyn Fn()>>,
    command_handlers: HashMap<char, Box<dyn Fn()>>,
}
//
// enum ExpectedKey {
//     //Command, implied by Init as Command can only be at the start of a vim sequence
//     Init,
//     // Count,
//     // Motion,
//     CountOrMotion,
// }

impl ToString for VimState {
    fn to_string(&self) -> String {
        let Self {
            command: cmd,
            count: c,
            motion: m,
            ..
        } = self;

        format!("{cmd}{c}{m}")
    }
}

impl VimState {
    pub fn new() -> Self {
        Self {
            command: String::new(),
            count: String::new(),
            motion: String::new(),
            //
            motion_handlers: HashMap::new(),
            command_handlers: HashMap::new(),
        }
    }

    pub fn register_motion(&mut self, c: char, action: Box<dyn Fn()>) {
        self.motion_handlers.insert(c, action);
    }

    pub fn register_command(&mut self, c: char, action: Box<dyn Fn()>) {
        self.command_handlers.insert(c, action);
    }

    pub fn push(&mut self, k: KeyCode) -> bool {
        let mut exit = false;

        match k {
            KeyCode::Esc => self.reset(),
            KeyCode::Char('q') => exit = true,
            KeyCode::Char(c) if MOTION_KEYS.contains(&c) => self.do_motion(c),
            KeyCode::Char(c) if COMMAND_KEYS.contains(&c) => self.set_command(c),
            KeyCode::Char(c) if NUMBER_KEYS.contains(&c) => self.add_count(c),
            _ => (),
        }

        let mut writer = stdout();
        let vim_sequence = self.to_string();

        let _ = crossterm::execute!(
            writer,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            Print(&vim_sequence)
        );

        exit
    }

    fn reset(&mut self) {
        *self = Self::new()
    }

    fn do_motion(&mut self, c: char) {
        self.motion = c.to_string();
        self.end()
    }
    fn set_command(&mut self, c: char) {
        self.command = c.to_string();
    }
    fn add_count(&mut self, c: char) {
        // match self.expected_key {
        //     ExpectedKey::CountOrMotion
        // }
        self.count.push(c);
    }

    fn end(&mut self) {
        self.reset()
    }
}

pub async fn main() {
    let mut reader = Reader::new();
    let mut rx = reader.init();

    let mut vim_state = VimState::new();

    let _ = crossterm::terminal::enable_raw_mode();
    loop {
        if let Some(Event::Key(KeyEvent { code, .. })) = rx.recv().await {
            let exit = vim_state.push(code);
            if exit {
                break;
            };
        }
    }
    let _ = crossterm::terminal::disable_raw_mode();
}
