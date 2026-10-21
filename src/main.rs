use crate::buffer::Buffer;
use crate::pager::Pager;
use crate::terminal::TerminalGuard;

mod action;
mod buffer;
mod cli;
mod pager;
mod terminal;

fn main() -> anyhow::Result<()> {
  let args = cli::Cli::parse()?;
  let buffer = Buffer::load(&args.input)?;
  let _guard = TerminalGuard::init()?;
  Pager::new(buffer)?.run()
}
