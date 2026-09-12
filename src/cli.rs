use std::path::PathBuf;

pub struct Cli {
    pub file: Option<PathBuf>,
}

impl Cli {
    pub fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut cli = Self { file: None };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("Usage: page-rs [FILE]");
                    std::process::exit(0);
                }
                file if !file.starts_with('-') => cli.file = Some(PathBuf::from(file)),
                _ => {}
            }
        }
        cli
    }
}
