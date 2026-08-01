use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{
    cursor::{Hide, Show, MoveTo}, 
    style::{Color, SetBackgroundColor, SetForegroundColor}, 
    execute
};
use std::io::stdout;

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Self {
        enable_raw_mode().expect("failed to enable raw mode");
        let _ = execute!(
            stdout(), 
            Clear(ClearType::All),
            Clear(ClearType::Purge),
            MoveTo(0, 0),
            Hide
        );
        TerminalGuard
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(), 
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset),
            Clear(ClearType::All), 
            Clear(ClearType::Purge),
            MoveTo(0,0), 
            Show
        );
    }
}