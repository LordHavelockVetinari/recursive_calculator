mod bool_gen;
mod compile;
mod ctrlc_handler;
mod environment;
mod expression;
mod garbage;
mod math;
mod parse;
mod program;
mod run;
#[cfg(test)]
mod test;

use crate::math::format::Format;
use crate::program::Program;
use clap::Parser;
use environment::Environment;
use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

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
    let args = Args::parse();
    if let Err(err) = run(&args) {
        println!("{}", err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let mut env = Environment::default();
    env.io_options.output_format = args.format;
    env.init_ctrlc_handler();
    let mut program = Program::new();
    env.io_options.are_errors_fatal = true;
    for lib in &args.load {
        run::run_file(lib, &mut program, &mut env)
            .map_err(|err| FailedToLoadLibrary(lib.clone(), err))?;
    }
    env.io_options.are_errors_fatal = false;
    if let Some(input) = &args.input {
        run::run_file(input, &mut program, &mut env)
    } else {
        run::repl(&mut program, &mut env)
    }
}
