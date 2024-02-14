use crate::environment::Environment;
use crate::math::format::Format;
use crate::{compile, parse};
use itertools::Itertools;
use std::fs;
use std::io::BufRead;
use std::path::Path;

#[test]
fn interpreter_test() {
    let input = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test_input.recalc"),
    )
    .unwrap();
    let expected_output = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("test_output.txt"),
    )
    .unwrap()
    .lines()
    .filter(|line| !line.trim_start().starts_with('#'))
    .join("\n");
    let mut output: Vec<u8> = vec![];
    let mut env = Environment::default();
    env.io_options.output = Box::new(&mut output);
    env.io_options.output_format = Format::Fraction;
    let code = parse::parse(&input).unwrap();
    let mut program = compile::compile(code).unwrap();
    program.run(&mut env).unwrap();
    drop(env);
    assert_eq!(output.lines().count(), expected_output.lines().count());
    for ((found, expected), i) in output.lines().zip(expected_output.lines()).zip(1..) {
        assert_eq!(found.unwrap(), expected, "Line {i}");
    }
}
