use crate::bool_gen::BoolGen;
use crate::ctrlc_handler::{CtrlCError, CtrlCHandler};
use crate::garbage;
use crate::math::format::{Format, FormattedValue};
use crate::math::Value;
use std::io::{self, BufRead, Write};

pub struct EvaluationEnvironemnt {
    bool_gen: BoolGen,
    ctrlc_handler: CtrlCHandler,
}

impl Default for EvaluationEnvironemnt {
    fn default() -> Self {
        Self {
            bool_gen: BoolGen::new(),
            ctrlc_handler: CtrlCHandler::mock(),
        }
    }
}

impl EvaluationEnvironemnt {
    pub fn gen_bool(&mut self) -> bool {
        self.bool_gen.gen()
    }

    pub fn tick(&mut self) -> Result<(), CtrlCError> {
        garbage::collect();
        self.ctrlc_handler.catch()
    }
}
pub struct IoOptions<'a> {
    pub input: Box<dyn BufRead + 'a>,
    pub output: Box<dyn Write + 'a>,
    pub error_output: Box<dyn Write + 'a>,
    pub output_format: Format,
    pub are_errors_fatal: bool,
    pub suggest_help: bool,
    pub show_welcome_message: bool,
}

impl<'a> Default for IoOptions<'a> {
    fn default() -> Self {
        Self {
            input: Box::new(std::io::stdin().lock()),
            output: Box::new(std::io::stdout()),
            error_output: Box::new(std::io::stderr()),
            output_format: Format::default(),
            are_errors_fatal: false,
            suggest_help: false,
            show_welcome_message: true,
        }
    }
}

#[derive(Default)]
pub struct Environment<'a> {
    pub evaluation_environment: EvaluationEnvironemnt,
    pub io_options: IoOptions<'a>,
}

impl<'a> Environment<'a> {
    pub fn input(&mut self) -> &mut (dyn BufRead + 'a) {
        &mut self.io_options.input
    }

    pub fn output(&mut self) -> &mut (dyn Write + 'a) {
        &mut self.io_options.output
    }

    pub fn error_output(&mut self) -> &mut (dyn Write + 'a) {
        &mut self.io_options.error_output
    }

    pub fn output_value(&mut self, value: &Value) -> Result<(), io::Error> {
        let fmt = self.io_options.output_format;
        writeln!(self.output(), "{}", FormattedValue(fmt, value))?;
        Ok(())
    }

    pub fn init_ctrlc_handler(&mut self) {
        self.evaluation_environment.ctrlc_handler = CtrlCHandler::new();
    }

    pub fn ignore_ctrlc(&mut self) {
        let _ = self.evaluation_environment.ctrlc_handler.catch();
    }
}
