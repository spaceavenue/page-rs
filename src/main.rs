use crate::buffer::Buffer;
use crate::terminal::TerminalGuard;

mod buffer;
mod cli;
mod terminal;

fn main() -> anyhow::Result<()> {
  let args = cli::Cli::parse()?;
  let buffer = Buffer::load(&args.input)?;
  let _guard = TerminalGuard::init()?;
  Ok(())
  // todo!("Nothing works just yet :p")
}
