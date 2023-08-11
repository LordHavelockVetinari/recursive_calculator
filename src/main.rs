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
use std::process::ExitCode;

use clap::Parser;
use math::format::Format;
use program::Environment;

#[derive(Parser)]
struct Args {
    ///Input file
    input: Option<String>,
    /// The format in which to display output
    #[arg(value_enum, default_value_t, short, long)]
    format: Format,
}

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
    if let Some(input) = &args.input {
        run::run_file(input, &mut env)
    } else {
        run::repl(&mut env)
    }
}
