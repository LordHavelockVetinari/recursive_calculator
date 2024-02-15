mod special_command;

use crate::environment::Environment;
use crate::program::{Program, ProgramError};
use crate::{compile, parse};
use std::error::Error;
use std::fs;
use std::ops::ControlFlow;
use std::path::Path;

fn maybe_suggest_help(env: &mut Environment) -> Result<(), Box<dyn Error>> {
    if env.io_options.suggest_help {
        writeln!(
            env.error_output(),
            "(For more information, type :help and press enter.)"
        )?;
    }
    Ok(())
}

fn run_str(code: &str, program: &mut Program, env: &mut Environment) -> Result<(), Box<dyn Error>> {
    env.ignore_ctrlc();
    let code = match parse::parse(code) {
        Ok(code) => code,
        Err(err) => {
            if env.io_options.are_errors_fatal {
                return Err(err.to_string().into());
            }
            writeln!(env.error_output(), "{err}")?;
            maybe_suggest_help(env)?;
            return Ok(());
        }
    };
    let backup = (!env.io_options.are_errors_fatal).then(|| program.clone());
    if let Err(err) = compile::compile_into(code, program) {
        if env.io_options.are_errors_fatal {
            return Err(err.into());
        }
        writeln!(env.io_options.error_output, "{err}")?;
        maybe_suggest_help(env)?;
        *program = backup.unwrap();
        return Ok(());
    }
    let res = program.run(env);
    match res {
        Ok(()) | Err(ProgramError::CtrlCError(_)) => Ok(()),
        Err(ProgramError::IoError(err)) => Err(err.into()),
    }
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
    env.io_options.suggest_help = true;
    if env.io_options.show_welcome_message {
        writeln!(
            env.output(),
            "Recursive Calculator {}!\n\
            Type :help and press enter for more information.",
            clap::crate_version!()
        )?;
    }
    loop {
        write!(env.output(), "recalc> ")?;
        env.output().flush()?;
        line_buf.clear();
        if env.input().read_line(&mut line_buf)? == 0 {
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
        let mut env = Environment::default();
        env.io_options.input = Box::new(input.as_bytes());
        env.io_options.output = Box::new(&mut output);
        env.io_options.error_output = Box::new(&mut error_output);
        env.io_options.show_welcome_message = false;
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
            "\
                zero = 0
                f(n) = zero^n - -n*f(n - 1)\n\
                f(5)\
            ",
            "\
                recalc> \
                recalc> \
                recalc> \
                120\n\
                recalc> \
            ",
            "",
        );
        #[cfg(not(miri))]
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
