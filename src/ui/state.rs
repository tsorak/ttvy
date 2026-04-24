use std::collections::VecDeque;

use ratatui::text::Line;

const MAX_BUFFER: usize = 1000;

pub struct AppState {
    messages: VecDeque<Line<'static>>,
    input: String,
    cursor: usize,
    last_sent: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            messages: VecDeque::with_capacity(128),
            input: String::new(),
            cursor: 0,
            last_sent: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_line(&mut self, line: Line<'static>) {
        if self.messages.len() == MAX_BUFFER {
            self.messages.pop_front();
        }
        self.messages.push_back(line);
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    pub fn visible_lines(&self, height: usize) -> Vec<Line<'static>> {
        if height == 0 {
            return Vec::new();
        }
        let total = self.messages.len();
        let take = total.min(height);
        let start = total - take;
        let padding = height - take;
        let mut lines: Vec<Line<'static>> = Vec::with_capacity(height);
        for _ in 0..padding {
            lines.push(Line::from(""));
        }
        lines.extend(self.messages.iter().skip(start).cloned());
        lines
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn cursor_display_col(&self) -> u16 {
        self.input[..self.cursor].chars().count() as u16
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some(c) = self.input[..self.cursor].chars().next_back() {
            let new_cursor = self.cursor - c.len_utf8();
            self.input.remove(new_cursor);
            self.cursor = new_cursor;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
        }
    }

    pub fn cursor_left(&mut self) {
        if let Some(c) = self.input[..self.cursor].chars().next_back() {
            self.cursor -= c.len_utf8();
        }
    }

    pub fn cursor_right(&mut self) {
        if let Some(c) = self.input[self.cursor..].chars().next() {
            self.cursor += c.len_utf8();
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor = self.input.len();
    }

    pub fn recall_last(&mut self) {
        if let Some(last) = &self.last_sent {
            self.input.clone_from(last);
            self.cursor = self.input.len();
        }
    }

    pub fn take_input(&mut self) -> String {
        let taken = std::mem::take(&mut self.input);
        self.cursor = 0;
        if !taken.is_empty() {
            self.last_sent = Some(taken.clone());
        }
        taken
    }
}
