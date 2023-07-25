use std::{
    io::{stdin, IsTerminal, Read},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::eyre::{bail, Result, WrapErr};
use tabled::Table;

#[derive(Parser, Debug)]
#[command(version, about, name = "KCL Compiler", long_about = None)]
struct Args {
    #[arg(long)]
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let Args { file } = Args::parse();
    let mut input = stdin();
    let mut source_code = String::new();
    if !input.is_terminal() {
        input
            .read_to_string(&mut source_code)
            .wrap_err("could not read from stdin")?;
    } else if let Some(file) = file {
        source_code = std::fs::read_to_string(&file)
            .wrap_err(format!("could not read {}", file.display()))?;
    } else {
        bail!("You must either supply a source code file via --file, or pipe source code in via stdin")
    }
    match compiler::parse(&source_code) {
        Ok((input, _ast)) if input.fragment().is_empty() => {
            println!("Successfully parsed your program")
        }
        Ok((remaining_input, _ast)) => {
            bail!("Part of your source code was not parsed: {remaining_input}")
        }
        Err(errors) => {
            eprintln!("Your program did not parse. Here is the chain of parser errors. This is similar to a stack trace: the top row is the deepest parser in the parse tree. The bottom row is the parse tree root.");
            let table = Table::new(errors);
            eprintln!("{table}");
            std::process::exit(1)
        }
    }
    Ok(())
}
