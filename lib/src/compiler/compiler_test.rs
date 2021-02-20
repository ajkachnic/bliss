use super::*;

use crate::lexer::Lexer;
use crate::object;
use crate::parser::Parser;
use code::{make, pretty};
use object::Object;

struct TestCase {
    input: &'static str,
    expected_constants: Vec<Object>,
    expected_instructions: Vec<Instructions>,
}

#[test]
fn test_arithmetic() {
    let tests: Vec<TestCase> = vec![
        TestCase {
            input: "1 + 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Add, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1; 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Pop, Vec::new()),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 - 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Sub, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 * 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Mul, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 / 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Div, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 % 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Mod, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "true",
            expected_constants: vec![],
            expected_instructions: vec![
                code::make(Opcode::True, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "false",
            expected_constants: vec![],
            expected_instructions: vec![
                code::make(Opcode::False, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
    ];

    run_tests(tests);
}

fn run_tests(tests: Vec<TestCase>) {
    for test in tests {
        let program = parse(test.input);
        let mut comp = Compiler::new();
        if let Err(error) = comp.compile(program) {
            panic!(error);
        }

        assert_eq!(test.expected_constants, comp.constants);
        let expected_instructions = test.expected_instructions.concat();

        let expected = pretty(expected_instructions);
        let actual = pretty(comp.instructions);

        assert_eq!(expected, actual);
    }
}

fn parse(input: &str) -> Program {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    parser.parse_program().unwrap()
}
