mod event;
mod state;

use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    widgets::Paragraph,
    Frame, Terminal,
};

pub use self::event::{handle as handle_event, AppEvent};
pub use self::state::AppState;

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

pub fn install_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        hook(info);
    }));
}

pub fn draw(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(frame.area());

    let visible = state.visible_lines(chunks[0].height as usize);
    frame.render_widget(Paragraph::new(visible), chunks[0]);

    let input_line = format!("> {}", state.input());
    frame.render_widget(Paragraph::new(input_line), chunks[1]);

    let cursor_x = chunks[1].x + 2 + state.cursor_display_col();
    frame.set_cursor_position((cursor_x, chunks[1].y));
}
