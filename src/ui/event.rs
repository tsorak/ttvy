use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::state::AppState;

pub enum AppEvent {
    Submit(String),
    Quit,
}

pub fn handle(event: Event, state: &mut AppState) -> Option<AppEvent> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(key, state),
        _ => None,
    }
}

fn handle_key(key: KeyEvent, state: &mut AppState) -> Option<AppEvent> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return Some(AppEvent::Quit);
    }

    match key.code {
        KeyCode::Enter => {
            let line = state.take_input();
            if line.is_empty() {
                None
            } else {
                Some(AppEvent::Submit(line))
            }
        }
        KeyCode::Char(c) => {
            state.insert_char(c);
            None
        }
        KeyCode::Backspace => {
            state.backspace();
            None
        }
        KeyCode::Delete => {
            state.delete();
            None
        }
        KeyCode::Left => {
            state.cursor_left();
            None
        }
        KeyCode::Right => {
            state.cursor_right();
            None
        }
        KeyCode::Home => {
            state.cursor_home();
            None
        }
        KeyCode::End => {
            state.cursor_end();
            None
        }
        KeyCode::Up => {
            state.recall_last();
            None
        }
        _ => None,
    }
}
