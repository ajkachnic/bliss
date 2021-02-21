use compiler::Compiler;
use object::Object;

use crate::ast::Program;
use crate::lexer::Lexer;
use crate::parser::Parser;

use super::*;

type TestCase = (&'static str, Object);

#[test]
fn test_arithmetic() {
    let tests: Vec<TestCase> = vec![
        ("1 + 2", Object::Number(3.0)),
        ("5 * 5 % 15", Object::Number(10.0)),
        ("1 * 5 + 3", Object::Number(8.0)),
        ("1 + 2 / 2", Object::Number(2.0)),
        ("1 / 5", Object::Number(0.2)),
        ("2 - 5", Object::Number(-3.0)),
    ];

    run_tests(tests);
}

#[test]
fn test_boolean() {
    let tests: Vec<TestCase> = vec![
        ("true", Object::Boolean(true)),
        ("false", Object::Boolean(false)),
    ];

    run_tests(tests);
}

fn run_tests(tests: Vec<TestCase>) {
    for (input, output) in tests {
        let program = parse(input);

        let mut comp = Compiler::new();
        if let Err(error) = comp.compile(program) {
            panic!(error);
        }

        let mut vm = VM::new(comp.into());
        if let Err(error) = vm.run() {
            panic!(error);
        }

        let stack_element = vm.last_stack_top();

        assert_eq!(Some(&output), stack_element);
    }
}

fn parse(input: &str) -> Program {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    parser.parse_program().unwrap()
}
