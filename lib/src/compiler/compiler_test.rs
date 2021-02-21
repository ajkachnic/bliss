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
            input: "-1",
            expected_constants: vec![Object::Number(1.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Minus, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "!true",
            expected_constants: vec![],
            expected_instructions: vec![
                code::make(Opcode::True, vec![]),
                code::make(Opcode::Bang, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
    ];

    run_tests(tests);
}

#[test]
fn test_boolean() {
    let tests = vec![
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
        TestCase {
            input: "1 > 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Greater, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 >= 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::GreaterEqual, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 < 2",
            expected_constants: vec![Object::Number(2.0), Object::Number(1.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Greater, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 <= 2",
            expected_constants: vec![Object::Number(2.0), Object::Number(1.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::GreaterEqual, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 == 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Equal, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
        TestCase {
            input: "1 != 2",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::NotEqual, Vec::new()),
                code::make(Opcode::Pop, Vec::new()),
            ],
        },
    ];

    run_tests(tests);
}

#[test]
fn test_conditionals() {
    let tests = vec![
        TestCase {
            input: "if true { 10 } else { 5 } 50",
            expected_constants: vec![Object::Number(10.0), Object::Number(5.0), Object::Number(50.0)],
            expected_instructions: vec![
                // 0000
                code::make(Opcode::True, vec![]),
                // 0001
                code::make(Opcode::JumpNotTruthy, vec![10]),
                // 0004
                code::make(Opcode::Constant, vec![0]),
                // 0007
                code::make(Opcode::Jump, vec![13]),
                // 0010
                code::make(Opcode::Constant, vec![1]),
                // 0013
                code::make(Opcode::Pop, vec![]),
                // 0014
                code::make(Opcode::Constant, vec![2]),
                // 0017
                code::make(Opcode::Pop, vec![]),
            ]
        }
    ];

    run_tests(tests);
}

#[test]
fn test_assignment_stmts() {
    let tests = vec![
        TestCase {
            input: "one = 1;
            two = 2;",
            expected_constants: vec![Object::Number(1.0), Object::Number(2.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::SetGlobal, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::SetGlobal, vec![1]),
            ]
        },
        TestCase {
            input: "one = 1;
            one;",
            expected_constants: vec![Object::Number(1.0)],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::SetGlobal, vec![0]),
                code::make(Opcode::GetGlobal, vec![0]),
                code::make(Opcode::Pop, vec![]),
            ]
        },
    ];

    run_tests(tests);
}

#[test]
fn test_strings() {
    let tests = vec![
        TestCase {
            input: "'foobar'",
            expected_constants: vec![Object::from("foobar")],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Pop, vec![])
            ]
        },
        TestCase {
            input: "'hello' + ' world!'",
            expected_constants: vec![
                Object::from("hello"),
                Object::from(" world!")
            ],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Add, vec![]),
                code::make(Opcode::Pop, vec![])
            ]
        },
    ];

    run_tests(tests);
}

#[test]
fn test_array() {
    let tests = vec![
        TestCase {
            input: "[]",
            expected_constants: vec![],
            expected_instructions: vec![
                code::make(Opcode::Array, vec![0]),
                code::make(Opcode::Pop, vec![])
            ]
        },
        TestCase {
            input: "[1, 2, 3]",
            expected_constants: vec![
                Object::Number(1.0),
                Object::Number(2.0),
                Object::Number(3.0),
            ],
            expected_instructions: vec![
                code::make(Opcode::Constant, vec![0]),
                code::make(Opcode::Constant, vec![1]),
                code::make(Opcode::Constant, vec![2]),
                code::make(Opcode::Array, vec![3]),
                code::make(Opcode::Pop, vec![])
            ]
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

        let expected_instructions = test.expected_instructions.concat();
        
        let expected = pretty(expected_instructions);
        let actual = pretty(comp.instructions);
        
        assert_eq!(expected, actual);
        assert_eq!(test.expected_constants, comp.constants);
    }
}

fn parse(input: &str) -> Program {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    parser.parse_program().unwrap()
}
