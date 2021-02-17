use std::fs;
use std::path;

use std::cell::RefCell;
use std::rc::Rc;

use lib::evaluation;
use lib::lexer::Lexer;
use lib::parser::Parser;
use lib::semantics;
use lib::style;
use path::Path;

// Executes a file
// The file should already be stat-ed to ensure we can access it
pub fn exec_file(path: &Path) -> std::io::Result<()> {
    let file = fs::read_to_string(path)?;

    let lexer = Lexer::new(&file);
    let mut parser = Parser::new(lexer);

    let program = match parser.parse_program() {
        Ok(program) => program,
        Err(error) => {
            handle_parser_error(error);
            return Ok(());
        }
    };
    let mut context = semantics::context::Context {
        ..Default::default()
    };
    let analysis = semantics::analyze::analyze_stmts(program.clone(), Some(&mut context));
    if let Err(errors) = analysis {
        handle_analysis_errors(errors);
        return Ok(());
    }

    let env = evaluation::env::Environment::new();

    let mut evaluator = evaluation::Evaluator::new(Rc::new(RefCell::new(env)));

    let result = evaluator.eval_program(program);
    match result {
        Ok(result) => {
            println!("{}", result);
        }
        Err(error) => eprintln!("{}", error),
    }

    Ok(())
}

fn handle_parser_error(error: String) {
    eprintln!(
        "{}\nWe had a few problems while parsing your code",
        style::bold("Parsing Errors:")
    );
    eprintln!("{}", error);
}

fn handle_analysis_errors(errors: Vec<String>) {
    eprintln!(
        "{}\nSorry to disturb you, but we had some trouble while analyzing your code for validity",
        style::bold("Semantic Analysis Errors:")
    );
    for error in errors {
        eprintln!("{}\n", error);
    }
}
