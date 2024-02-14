use std::error::Error;
use std::fs;
use std::ops::ControlFlow;
use std::str::FromStr;

use crate::environment::Environment;
use crate::math::format;
use crate::program::Program;

pub type Result = std::result::Result<ControlFlow<()>, Box<dyn Error>>;

trait Command {
    fn run(&self, program: &mut Program, env: &mut Environment) -> Result;
}

struct Quit;

impl Command for Quit {
    fn run(&self, _program: &mut Program, _env: &mut Environment) -> Result {
        Ok(ControlFlow::Break(()))
    }
}

struct Format {
    new_format: String,
}

impl Command for Format {
    fn run(&self, _program: &mut Program, env: &mut Environment) -> Result {
        if self.new_format.is_empty() {
            let fmt = env.io_options.output_format;
            writeln!(env.output(), "The current format is: {}.", fmt,)?;
            writeln!(
                env.output(),
                "Type :format fraction, :format mixed or :format scientific to change it."
            )?;
            return Ok(ControlFlow::Continue(()));
        }
        match format::Format::from_str(&self.new_format) {
            Ok(fmt) => env.io_options.output_format = fmt,
            Err(err) => writeln!(env.error_output(), "{err}")?,
        };
        Ok(ControlFlow::Continue(()))
    }
}

struct Delete {
    name: String,
}

impl Command for Delete {
    fn run(&self, program: &mut Program, env: &mut Environment) -> Result {
        if program.undefine(&self.name).is_err() {
            writeln!(
                env.error_output(),
                "no constant or function named \"{}\"",
                self.name
            )?;
        }
        Ok(ControlFlow::Continue(()))
    }
}

struct Load {
    file: String,
}

impl Command for Load {
    fn run(&self, program: &mut Program, env: &mut Environment) -> Result {
        let code = match fs::read_to_string(&self.file) {
            Ok(code) => code,
            Err(err) => {
                writeln!(env.error_output(), "{err}")?;
                return Ok(ControlFlow::Continue(()));
            }
        };
        super::run_str(&code, program, env)?;
        Ok(ControlFlow::Continue(()))
    }
}

struct Help;

impl Command for Help {
    fn run(&self, _program: &mut Program, env: &mut Environment) -> Result {
        write!(env.output(), include_str!("help.txt"))?;
        Ok(ControlFlow::Continue(()))
    }
}

pub fn run(command: &str, program: &mut Program, env: &mut Environment) -> Result {
    let command = command.trim().trim_start_matches(':').trim_start();
    let (name, args) = command
        .split_once(char::is_whitespace)
        .unwrap_or((command, ""));
    let name = name.to_lowercase();
    let args = args.trim_start().to_string();
    let cmd: Box<dyn Command> = match &name[..] {
        "q" | "quit" => Box::new(Quit),
        "f" | "format" => Box::new(Format { new_format: args }),
        "d" | "delete" => Box::new(Delete { name: args }),
        "l" | "load" => Box::new(Load { file: args }),
        "h" | "help" => Box::new(Help),
        _ => {
            writeln!(env.error_output(), "unknown command: \":{}\"", name)?;
            super::maybe_suggest_help(env)?;
            return Ok(ControlFlow::Continue(()));
        }
    };
    cmd.run(program, env)
}
