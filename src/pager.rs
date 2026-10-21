use std::io::{self, Stdout, Write};
use std::time::Duration;

use crossterm::cursor::MoveTo;
use crossterm::event::Event;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{event, queue, terminal};

use crate::action::{Action, action_for_key};
use crate::buffer::Buffer;

const STREAM_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub struct Pager {
  buffer: Buffer,
  top: usize,
  width: u16,
  height: u16,
  running: bool,
  dirty: bool,
  follow: bool,
}

impl Pager {
  pub fn new(buffer: Buffer) -> anyhow::Result<Self> {
    let (width, height) = terminal::size()?;

    Ok(Self {
      buffer,
      top: 0,
      width,
      height,
      running: true,
      dirty: true,
      follow: false,
    })
  }

  fn body_height(&self) -> usize {
    self.height.saturating_sub(1) as usize
  }

  pub fn run(&mut self) -> anyhow::Result<()> {
    let mut stdout = io::stdout();

    while self.running {
      let got_data = self.buffer.read_from_stdin()?;
      if got_data {
        self.dirty = true;
        if self.follow {
          self.jump_to_bottom();
        }
      }

      if self.dirty {
        self.render(&mut stdout)?;
        self.dirty = false;
      }

      if self.buffer.is_streaming() {
        if event::poll(STREAM_POLL_INTERVAL)? {
          self.handle_event(event::read()?);
        }
      } else {
        self.handle_event(event::read()?);
      }
    }

    Ok(())
  }

  fn handle_event(&mut self, event: Event) {
    match event {
      Event::Key(key_event) => {
        if let Some(action) = action_for_key(key_event) {
          self.dispatch(action);
        }
      }
      Event::Resize(w, h) => {
        (self.width, self.height) = (w, h);
        self.dirty = true;
      }
      _ => {}
    }
  }

  fn dispatch(&mut self, action: Action) {
    use Action::{
      GotoBottom, GotoTop, HalfPageDown, HalfPageUp, LineDown, LineUp, PageDown, PageUp, Quit,
    };
    let body = self.body_height().cast_signed();
    match action {
      LineUp => self.scroll_by(-1),
      LineDown => self.scroll_by(1),
      PageUp => self.scroll_by(-body),
      PageDown => self.scroll_by(body),
      HalfPageUp => self.scroll_by(-(body / 2).max(1)),
      HalfPageDown => self.scroll_by((body / 2).max(1)),
      GotoTop => {
        self.follow = false;
        self.set_top(0);
      }
      GotoBottom => self.jump_to_bottom(),
      Quit => self.running = false,
    }
    self.dirty = true;
  }

  fn scroll_by(&mut self, amount: isize) {
    if amount < 0 {
      self.follow = false;
    }

    let new_top = (self.top.cast_signed() + amount).max(0).cast_unsigned();
    self.set_top(new_top);
  }

  fn set_top(&mut self, top: usize) {
    self.index_near(top);
    let total = self.buffer.line_count();
    let max_top = total.saturating_sub(self.body_height());
    self.top = top.min(max_top);
  }

  fn jump_to_bottom(&mut self) {
    self.buffer.index_till_line(usize::MAX);
    let total = self.buffer.line_count();
    self.top = total.saturating_sub(self.body_height());
    self.follow = self.buffer.is_streaming();
  }

  fn index_near(&mut self, top: usize) {
    let last_row = top + self.body_height();
    self.buffer.index_till_line(last_row);
  }

  fn render(&mut self, stdout: &mut Stdout) -> anyhow::Result<()> {
    self.index_near(self.top);
    let body = self.body_height();
    for row in 0..body {
      queue!(stdout, MoveTo(0, row as u16), Clear(ClearType::CurrentLine))?;
      match self.buffer.line_bytes(self.top + row) {
        Some(bytes) => {
          let text = String::from_utf8_lossy(bytes);
          let truncated = text.chars().take(self.width as usize).collect::<String>();
          queue!(stdout, Print(truncated))?;
        }
        None => {
          queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print("~"),
            ResetColor
          )?;
        }
      }
    }

    self.render_status(stdout)?;
    stdout.flush()?;
    Ok(())
  }

  fn render_status(&self, stdout: &mut Stdout) -> anyhow::Result<()> {
    let total = self.buffer.line_count();
    let still_reading = self.buffer.is_streaming();
    let percent = if total == 0 {
      100
    } else {
      (self.top.min(total) * 100 / total.max(1)).min(100)
    };

    let status = format!(
      "{} {}{} lines {}% {}",
      self.buffer.filename(),
      total,
      if still_reading { "+" } else { "" },
      percent,
      if still_reading { "[reading...]" } else { "" }
    );

    queue!(
      stdout,
      MoveTo(0, self.height.saturating_sub(1)),
      Clear(ClearType::CurrentLine),
      SetForegroundColor(Color::Black),
      SetBackgroundColor(Color::DarkGrey),
      Print(format!("{:<width$}", status, width = self.width as usize)),
      ResetColor
    )?;
    Ok(())
  }
}
