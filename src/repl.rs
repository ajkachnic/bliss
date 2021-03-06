use evaluation::Evaluator;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::cell::RefCell;
use std::rc::Rc;

use lib::evaluation;
use lib::lexer;
use lib::parser::Parser;
use lib::style;

pub fn start() {
    let mut rl = Editor::<()>::new();
    let env = evaluation::env::Environment::new();
    let mut evaluator = evaluation::Evaluator::new(Rc::new(RefCell::new(env)));
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                eval(line.as_str() /* &mut context*/, &mut evaluator)
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

fn eval(line: &str, /*context: &mut Context,*/ eval: &mut Evaluator) {
    let l = lexer::Lexer::new(line);
    let mut p = Parser::new(l, line.to_string());

    let program = p.parse_program();
    if let Ok(program) = program {
        // let analysis = semantics::analyze::analyze_stmts(program.clone(), Some(context));
        // match Ok(true) {
        // Ok(_) => {
        let evaled = eval.eval_program(program);
        if let Ok(evaled) = evaled {
            println!("{}", evaled);
        } else if let Err(error) = evaled {
            println!("An error occurred while evaluating your code:\n{}", error);
        }
    } else if let Err(error) = program {
        println!(
            "{}\nWe had a problem while parsing your code",
            style::bold("Parse Error:")
        );
        println!("{}", error);
    }
}
