use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
  Stdin,
  Path(PathBuf),
}

impl std::fmt::Display for Input {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Stdin => write!(f, "[stdin]"),
      Self::Path(p) => write!(f, "{}", p.display()),
    }
  }
}

pub struct Cli {
  pub input: Input,
}

impl Cli {
  pub fn parse() -> anyhow::Result<Self> {
    let args = std::env::args().skip(1);
    let mut raw_input = None;

    for arg in args {
      match arg.as_str() {
        "-h" | "--help" => {
          println!("Usage: page-rs [- | FILE]");
          std::process::exit(0);
        }

        "-" => raw_input = Some(Input::Stdin),
        file if !file.starts_with('-') => raw_input = Some(Input::Path(PathBuf::from(file))),
        flag => anyhow::bail!("unknown option/argument: {flag}"),
      }
    }
    let input = if let Some(input) = raw_input {
      input
    } else {
      // check if stdin has data or is an interactive terminal if no argument was passed.
      anyhow::ensure!(
        !std::io::stdin().is_terminal(),
        "missing filename or piped input."
      );
      Input::Stdin
    };
    Ok(Self { input })
  }
}
