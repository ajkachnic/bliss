use evaluation::Evaluator;
use lexer::Lexer;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::rc::Rc;
use std::{cell::RefCell, process};

use lib::{code::pretty, compiler::Bytecode, lexer};
use lib::parser::Parser;
use lib::semantics;
use lib::style;
use lib::vm::VM;
use lib::{ast::Program, compiler::Compiler, evaluation};
use semantics::context::Context;

pub fn start() {
    let mut rl = Editor::<()>::new();
    let mut context = Context {
        ..Default::default()
    };
    let env = evaluation::env::Environment::new();
    let mut evaluator = evaluation::Evaluator::new(Rc::new(RefCell::new(env)));
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                eval(line.as_str(), &mut context, &mut evaluator)
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn parse(source: &str) -> Program {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);

    let program = parser.parse_program();

    match program {
        Ok(program) => program,
        Err(error) => {
            println!(
                "{}\nWe had a few problems while parsing your code",
                style::bold("Parsing Errors:")
            );
            println!("{}", error);
            process::exit(1);
        }
    }
}

fn eval(line: &str, context: &mut Context, eval: &mut Evaluator) {
    let program = parse(line);
    let analysis = semantics::analyze::analyze_stmts(program.clone(), Some(context));
    match analysis {
        Ok(_) => {
            let mut compiler = Compiler::new();
            
            compiler.compile(program).unwrap();
            let bytecode: Bytecode = compiler.into();
            println!("{}", pretty(bytecode.instructions.clone()));
            let mut vm = VM::new(bytecode);
            vm.run().unwrap();

            let top = vm.last_stack_top();
            if let Some(evaled) = top {
                println!("{}", evaled);
            }
            // else if let Err(error) = evaled {
            //     println!("An error occurred while evaluating your code:\n{}", error);
            // }
        }
        Err(errors) => {
            println!(
                    "{}\nSorry to disturb you, but we had some trouble while analyzing your code for validity",
                     style::bold("Semantic Analysis Errors:")
                );
            for error in errors {
                println!("{}\n", error);
            }
        }
    }
}
