mod special_command;

use crate::program::{Environment, Program};
use crate::{compile, parse};
use std::error::Error;
use std::fs;
use std::ops::ControlFlow;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct ErrorMessage(String);

fn maybe_suggest_help(env: &mut Environment) -> Result<(), Box<dyn Error>> {
    if env.suggest_help {
        writeln!(
            env.error_output,
            "(For more information, type :help and press enter.)"
        )?;
    }
    Ok(())
}

fn run_str(code: &str, program: &mut Program, env: &mut Environment) -> Result<(), Box<dyn Error>> {
    let code = match parse::parse(code) {
        Ok(code) => code,
        Err(err) => {
            if env.are_errors_fatal {
                return Err(ErrorMessage(err.to_string()).into());
            }
            writeln!(env.error_output, "{err}")?;
            maybe_suggest_help(env)?;
            return Ok(());
        }
    };
    let backup = (!env.are_errors_fatal).then(|| program.clone());
    if let Err(err) = compile::compile_into(code, program) {
        if env.are_errors_fatal {
            return Err(err.into());
        }
        writeln!(env.error_output, "{err}")?;
        maybe_suggest_help(env)?;
        *program = backup.unwrap();
        return Ok(());
    }
    program.run(env)?;
    Ok(())
}

pub fn run_file(
    filename: &Path,
    program: &mut Program,
    env: &mut Environment,
) -> Result<(), Box<dyn Error>> {
    let code = fs::read_to_string(filename)?;
    run_str(&code, program, env)
}

pub fn repl(program: &mut Program, env: &mut Environment) -> Result<(), Box<dyn Error>> {
    let mut line_buf = String::new();
    env.suggest_help = true;
    if env.show_welcome_message {
        writeln!(
            env.output,
            "Recursive Calculator {}!\n\
            Type :help and press enter for more information.",
            clap::crate_version!()
        )?;
    }
    loop {
        write!(env.output, "recalc> ")?;
        env.output.flush()?;
        line_buf.clear();
        if env.input.read_line(&mut line_buf)? == 0 {
            break;
        };
        if line_buf.trim_start().starts_with(':') {
            match special_command::run(&line_buf, program, env)? {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(()) => continue,
            }
        }
        run_str(&line_buf, program, env)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_repl(input: &str, expected_output: &str, expected_error_output: &str) {
        let mut output = Vec::<u8>::new();
        let mut error_output = Vec::<u8>::new();
        let mut env = Environment {
            input: Box::new(input.as_bytes()),
            output: Box::new(&mut output),
            error_output: Box::new(&mut error_output),
            show_welcome_message: false,
            ..Environment::default()
        };
        repl(&mut Program::new(), &mut env).unwrap();
        drop(env);
        assert_eq!(std::str::from_utf8(&output).unwrap(), expected_output);
        assert_eq!(
            std::str::from_utf8(&error_output).unwrap(),
            expected_error_output
        );
    }

    #[test]
    fn repl_test() {
        assert_repl(
            "1 + 1",
            "\
                recalc> \
                2\n\
                recalc> \
            ",
            "",
        );
        assert_repl(
            "\
                x = f(y) = 3\n\
                f(x) + 3*x\n\
            ",
            "\
                recalc> \
                recalc> \
                12\n\
                recalc> \
            ",
            "",
        );
        assert_repl(
            &format!(
                "\
                    :l {test_file}\n\
                    y = 3\n\
                    :l {test_file}\n\
                    y = 4\n\
                    :l {test_file}\n\
                    f(123)\
                ",
                test_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("resources")
                    .join("run_test.recalc")
                    .display()
            ),
            "\
                recalc> \
                recalc> \
                recalc> \
                3\n\
                recalc> \
                recalc> \
                4\n\
                recalc> \
                4\n\
                recalc> \
            ",
            "constant not found: y\n(For more information, type :help and press enter.)\n",
        )
    }
}
