use std::io::{self, Stdout, Write, stdout};

use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};

pub struct TerminalGuard {
    _private: (),
}

impl TerminalGuard {
    pub fn init() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        // panic hook to restore terminal on panic
        let default_panic = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = Self::restore_terminal(&mut io::stdout());
            default_panic(info);
        }));

        Ok(Self { _private: () })
    }

    fn restore_terminal(stdout: &mut Stdout) -> io::Result<()> {
        execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        stdout.flush()?;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = stdout();
        let _ = Self::restore_terminal(&mut stdout);
    }
}
