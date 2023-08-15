mod bool_gen;
mod compile;
mod expression;
mod math;
mod parse;
mod program;
mod run;
#[cfg(test)]
mod test;

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use math::format::Format;
use program::{Environment, Program};

#[derive(Parser)]
struct Args {
    /// Input file
    input: Option<PathBuf>,
    /// The format in which to display output
    #[arg(value_enum, default_value_t, short, long)]
    format: Format,
    /// Load a library
    #[arg(short, long)]
    load: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to load library {0:?}:\n{1}")]
struct FailedToLoadLibrary(PathBuf, Box<dyn Error>);

fn main() -> ExitCode {
    let _ = ctrlc::set_handler(|| std::process::exit(0));
    let args = Args::parse();
    if let Err(err) = run(&args) {
        println!("{}", err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let mut env = Environment {
        output_format: args.format,
        ..Default::default()
    };
    let mut program = Program::new();
    env.are_errors_fatal = true;
    for lib in &args.load {
        run::run_file(lib, &mut program, &mut env)
            .map_err(|err| FailedToLoadLibrary(lib.clone(), err))?;
    }
    env.are_errors_fatal = false;
    if let Some(input) = &args.input {
        run::run_file(input, &mut program, &mut env)
    } else {
        run::repl(&mut program, &mut env)
    }
}
