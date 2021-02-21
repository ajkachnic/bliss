use evaluation::Evaluator;
use lexer::Lexer;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::{borrow::BorrowMut, rc::Rc};
use std::{cell::RefCell, process};

use lib::{code::pretty, compiler::{Bytecode, symbol_table::Symbol}, lexer, object::Object};
use lib::parser::Parser;
use lib::semantics;
use lib::style;
use lib::vm::VM;
use lib::compiler::symbol_table::SymbolTable;
use lib::{ast::Program, compiler::Compiler, evaluation};
use semantics::context::Context;

struct State {
    symbols: SymbolTable,
    constants: Vec<Object>,
    globals: Vec<Object>,
}

pub fn start() {
    let mut rl = Editor::<()>::new();
    let mut context = Context {
        ..Default::default()
    };
    let mut state = State {
        symbols: SymbolTable::new(),
        constants: vec![],
        globals: vec![]
    };

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                eval(line.as_str(), &mut context, &mut state)
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

fn eval(line: &str, context: &mut Context, state: &mut State) {
    let program = parse(line);
    let analysis = semantics::analyze::analyze_stmts(program.clone(), Some(context));
    match analysis {
        Ok(_) => {
            let mut compiler = Compiler::new_with_state(state.symbols.to_owned(), state.constants.to_owned());
            
            compiler.compile(program).unwrap();
            let bytecode: Bytecode = compiler.clone().into();

            state.constants = bytecode.constants.to_owned();
            state.symbols = compiler.symbols;
            let mut vm = VM::new_with_globals(bytecode, state.globals.to_owned());
            vm.run().unwrap();

            let top = vm.last_stack_top();
            if let Some(evaled) = top {
                println!("{}", evaled);
            }

            state.globals = vm.globals;
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
