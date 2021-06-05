use super::*;
use crate::ast::{Expr, Ident, Stmt};
use crate::lexer::Lexer;

#[test]
fn test_assign_stmt() {
    let input = "
	let x = 5;
	let y = 10;
	let foobar = 838383;
	";
    let expected = vec![
        Stmt::Assign(Ident::from("x").into(), Expr::Number(5.0)),
        Stmt::Assign(Ident::from("y").into(), Expr::Number(10.0)),
        Stmt::Assign(Ident::from("foobar").into(), Expr::Number(838383.0)),
    ];
    test_output(input, expected)
}
#[test]
fn test_return_stmt() {
    let input = "
	return 5;
	return 10;
	return 993322;
	";
    let expected = vec![
        Stmt::Return(Expr::Number(5.0)),
        Stmt::Return(Expr::Number(10.0)),
        Stmt::Return(Expr::Number(993322.0)),
    ];
    test_output(input, expected);
}

#[test]
fn test_import_stmt() {
    let input = "
    import foo from 'foo';
    import bar from \"foobar\";
	";
    let expected = vec![
        Stmt::Import {
            name: Ident("foo".to_string()).into(),
            source: Expr::String("foo".to_string()),
        },
        Stmt::Import {
            name: Ident("bar".to_string()).into(),
            source: Expr::String("foobar".to_string()),
        },
    ];
    test_output(input, expected)
}
#[test]
fn test_identifier_expression() {
    let input = "foobar;";
    let expected: Vec<Stmt> = vec![Stmt::Expr(Ident::from("foobar").into())];
    test_output(input, expected)
}
#[test]
fn test_string_expression() {
    let input = "'foobar' \"barfoo\"";
    let expected = vec![
        Expr::String("foobar".to_string()).into(),
        Expr::String("barfoo".to_string()).into(),
    ];
    test_output(input, expected)
}
#[test]
fn test_symbol_expression() {
    let input = ":bar :foo";
    let expected = vec![
        Expr::Symbol("bar".to_string()).into(),
        Expr::Symbol("foo".to_string()).into(),
    ];
    test_output(input, expected)
}
#[test]
fn test_number_expression() {
    let input = "5;";
    let expected = Expr::Number(5.0).into();
    test_output(input, expected)
}

#[test]
fn test_prefix_expression() {
    let inputs = vec!["-5", "!5"];
    let outputs: Vec<Vec<Stmt>> = vec![
        Expr::Prefix(String::from("-"), Box::new(Expr::Number(5.0))).into(),
        Expr::Prefix(String::from("!"), Box::new(Expr::Number(5.0))).into(),
    ];
    inputs.iter().enumerate().for_each(|(index, input)| {
        let expected = outputs[index].clone();
        test_output(input, expected)
    });
}
#[test]
fn test_boolean_expression() {
    let input = "true; false;";
    let expected = vec![Expr::Boolean(true).into(), Expr::Boolean(false).into()];
    test_output(input, expected)
}

#[test]
fn test_grouped_expressions() {
    let input = "1 * 5 + (5 / 2)";
    let expected = vec![Expr::Infix(
        Box::new(Expr::Infix(
            Box::new(Expr::Number(1.0)),
            String::from("*"),
            Box::new(Expr::Number(5.0)),
        )),
        String::from("+"),
        Box::new(Expr::Infix(
            Box::new(Expr::Number(5.0)),
            String::from("/"),
            Box::new(Expr::Number(2.0)),
        )),
    )
    .into()];
    test_output(input, expected)
}

#[test]
fn test_if_expression() {
    let input = "if true { 10 } else { 5 }";
    let expected = vec![Expr::If {
        condition: Box::new(Expr::Boolean(true)),
        consequence: Expr::Number(10.0).into(),
        alternative: Expr::Number(5.0).into(),
    }
    .into()];
    test_output(input, expected)
}

#[test]
fn test_function_expression() {
    let input = "fn(foo, bar) -> {
        foo + bar
    }";
    let expected = vec![Expr::Function {
        parameters: vec![Ident::from("foo"), Ident::from("bar")],
        body: Expr::Infix(
            Box::new(Ident::from("foo").into()),
            String::from("+"),
            Box::new(Ident::from("bar").into()),
        )
        .into(),
    }
    .into()];
    test_output(input, expected)
}

#[test]
fn test_array_expression() {
    let inputs = vec!["[ 1, 2, 3, 4]", "[[ true, false ]]", "[[ :ok, 10 ]]"];
    let outputs: Vec<Vec<Stmt>> = vec![
        Expr::Array(vec![
            Expr::Number(1.0),
            Expr::Number(2.0),
            Expr::Number(3.0),
            Expr::Number(4.0),
        ])
        .into(),
        Expr::Array(vec![Expr::Array(vec![
            Expr::Boolean(true),
            Expr::Boolean(false),
        ])])
        .into(),
        Expr::Array(vec![Expr::Array(vec![
            Expr::Symbol("ok".to_string()),
            Expr::Number(10.0),
        ])])
        .into(),
    ];
    inputs.iter().enumerate().for_each(|(index, input)| {
        let expected = outputs[index].clone();
        test_output(input, expected)
    });
}

#[test]
fn test_hash_expression() {
    let input = "{ name = 'bob', age = 15, height, status = :online }";
    let expected = Expr::Hash(vec![
        (Ident::from("name"), Expr::from("bob")),
        (Ident::from("age"), Expr::from(15.0)),
        (Ident::from("height"), Ident::from("height").into()),
        (Ident::from("status"), Expr::Symbol("online".to_string())),
    ])
    .into();
    test_output(input, expected)
}

#[test]
fn test_call_expression() {
    let input = "foobar(a, b, c)";
    let expected = Expr::Call {
        function: Box::new(Ident::from("foobar").into()),
        arguments: vec![
            Ident::from("a").into(),
            Ident::from("b").into(),
            Ident::from("c").into(),
        ],
    }
    .into();
    test_output(input, expected)
}

#[test]
fn test_pattern() {
    let input = "[ 4.5, foo, true, :bar, 'hello', { abc, def }, _ ]";
    let expected = Pattern::Array(vec![
        Pattern::Number(4.5),
        Pattern::Ident(Ident::from("foo")),
        Pattern::Boolean(true),
        Pattern::Symbol("bar".to_string()),
        Pattern::String("hello".to_string()),
        Pattern::Hash(vec![(Ident::from("abc"), None), (Ident::from("def"), None)]),
        Pattern::Nothing,
    ]);

    // Specialized check
    fn test_output(input: &str, expected: Pattern) {
        let l = Lexer::new(input);
        let mut p = Parser::new(l);
        let pattern = p.parse_pattern();
        if let Ok(pattern) = pattern {
            assert_eq!(pattern, expected)
        } else if let Err(err) = pattern {
            println!("Parser had an error:\n{}", err);
            assert!(false);
        }
    }

    test_output(input, expected)
}

#[test]
fn test_match_expression() {
    let input = "true :: {
        true -> 1 + 1,
        false -> 2 + 2
    }";
    let expected = Expr::Match {
        condition: Expr::from(true).into(),
        cases: vec![
            (
                Pattern::Boolean(true),
                Expr::Infix(
                    Box::new(Expr::from(1.0)),
                    String::from("+"),
                    Box::new(Expr::from(1.0)),
                )
                .into(),
            ),
            (
                Pattern::Boolean(false),
                Expr::Infix(
                    Box::new(Expr::from(2.0)),
                    String::from("+"),
                    Box::new(Expr::from(2.0)),
                )
                .into(),
            ),
        ],
    }
    .into();
    test_output(input, expected);
}

fn test_output(input: &str, expected: Vec<Stmt>) {
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    if let Ok(program) = program {
        check_program(program, expected);
    } else if let Err(err) = program {
        println!("Parser had an error:\n{}", err);
        assert!(false);
    }
}

fn check_program(program: Program, expected: Vec<Stmt>) {
    expected.iter().enumerate().for_each(|(index, stmt)| {
        assert_eq!(stmt, &program.0[index]);
    });
}
