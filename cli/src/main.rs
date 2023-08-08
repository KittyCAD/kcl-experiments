use std::{
    io::{stdin, IsTerminal, Read},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::eyre::{bail, Result, WrapErr};
use tabled::{Table, Tabled};

#[derive(Parser, Debug)]
#[command(version, about, name = "KCL Compiler", long_about = None)]
struct Args {
    #[arg(long)]
    file: Option<PathBuf>,
}

impl Args {
    /// Read a KCL file from wherever the user said to.
    fn read_input_source_code(&self) -> Result<String> {
        let mut input = stdin();
        let mut source_code = String::new();
        if !input.is_terminal() {
            input
                .read_to_string(&mut source_code)
                .wrap_err("could not read from stdin")?;
            Ok(source_code)
        } else if let Some(ref file) = self.file {
            std::fs::read_to_string(file).wrap_err(format!("could not read {}", file.display()))
        } else {
            bail!("You must either supply a source code file via --file, or pipe source code in via stdin")
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let source_code = args.read_input_source_code()?;

    // Parse the KCL file and print results.
    match compiler::parse(&source_code) {
        Ok((input, ast)) if input.fragment().is_empty() => {
            print_program_analysis(ast);
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

fn print_program_analysis(ast: compiler::AbstractSyntaxTree<'_>) {
    println!("Successfully parsed your program. These are its functions.");

    #[derive(Tabled)]
    struct Row<'a> {
        #[tabled(rename = "Name")]
        function_name: &'a str,
        line: u32,
        columns: String,
    }
    let tbl = tabled::Table::new(ast.all_functions().map(
        |(
            function_name,
            compiler::semantics::SourceRange {
                start_line,
                start_column,
                length,
            },
        )| Row {
            function_name,
            line: start_line,
            columns: format!("{start_column}, {}", start_column + length),
        },
    ));
    println!("{tbl}");
}
