use super::{context::Context, util};
use crate::ast::{Expr, Program, Stmt, Pattern};
use crate::style::{bold, yellow};
use std::default::Default;

type AnalysisResult = Result<(), Vec<String>>;

pub fn analyze_stmts(program: Program, parent: Option<&mut Context>) -> AnalysisResult {
    let mut default = Context {
        ..Default::default()
    };
    let mut context = match parent {
        Some(ctx) => ctx,
        None => &mut default,
    };
    let mut results = vec![];
    for stmt in program.0 {
        let analysis = analyze_stmt(stmt, &mut context);
        if let Err(errors) = analysis {
            for err in errors {
                results.push(err)
            }
        }
    }
    if !results.is_empty() {
        return Err(results);
    }
    Ok(())
}

pub fn analyze_stmt(stmt: Stmt, context: &mut Context) -> AnalysisResult {
    match stmt {
        Stmt::Expr(expr) => analyze_expr(expr, context),
        Stmt::Assign(name, expr) => {
            let mut errors = vec![];
            if let Pattern::Array(array) = name {
                for item in array {
                    if let Pattern::Ident(item) = item {
                        context.add(item.0, expr.clone());
                    } else {
                        errors.push(
                            "Attempted to pattern match with non identifier value".to_string(),
                        )
                    }
                }
            } else if let Pattern::Hash(hash) = name {
                for (_key, value) in hash {
                    if let Pattern::Ident(value) = value {
                        context.add(value.0, expr.clone());
                    } else {
                        errors.push(
                            "Attempted to pattern match with non identifier value".to_string(),
                        )
                    }
                }
            } else if let Pattern::Ident(ident) = name {
                context.add(ident.0, expr.clone());
            }
            let res = analyze_expr(expr, context);
            interpolate_errors(res, &mut errors);
            if !errors.is_empty() {
                return Err(errors);
            }
            Ok(())
        }
        Stmt::Return(_) => Ok(()),
        Stmt::Import { name, source } => {
            let mut errors = vec![];
            if !util::is_ident(&name) {
                errors.push(
          format!("While analyzing an import statement, we were expecting to find the name as an identifier, but we instead found this: {}
          
Hint: Only use identifiers as names in imports
Here is an example of a valid import statement:
import ident from 'string';", &name)
        )
            }
            if !util::is_string(&source) {
                errors.push(
          format!("While analyzing an import statement, we were expecting to find a string as the import source, but we instead found {}
          
Hint: Imports in bliss as static, meaning that you can't use other forms of expressions to determine the source. This is to simplify our handling of imports", &source)
        )
            }
            if !errors.is_empty() {
                return Err(errors);
            }
            Ok(())
        }
    }
}

pub fn analyze_expr(expr: Expr, context: &mut Context) -> AnalysisResult {
    let mut errors = vec![];
    match expr {
        Expr::Ident(ident) => {
            if !context.has(ident.0.clone()) {
                errors.push(format!(
                    "Identifier {} used before declaration",
                    bold(&yellow(&ident.0))
                ))
            }
        }
        Expr::If {
            condition,
            consequence,
            alternative,
        } => {
            let res = analyze_expr(*condition, context);
            interpolate_errors(res, &mut errors);
            let res = analyze_stmts(consequence, Some(&mut Context::new_child_block(context)));
            interpolate_errors(res, &mut errors);
            let res = analyze_stmts(alternative, Some(&mut Context::new_child_block(context)));
            interpolate_errors(res, &mut errors);
        }
        // Expr::Call {
        //     function,
        //     arguments,
        // } => {
        //     if util::is_function(&*function) || util::is_ident(&*function) {
        //         let res =
        //             analyze_expr(*function.clone(), &mut Context::new_function_block(context));
        //         interpolate_errors(res, &mut errors);
        //     } else {
        //         errors.push(
        //             format!("While parsing a call expression, we expected a function literal or an identifier, but we found {}, which is not callable", function)
        //         )
        //     }
        //     for arg in arguments {
        //         let res = analyze_expr(arg, context);
        //         interpolate_errors(res, &mut errors);
        //     }
        // }
        Expr::Array(items) => {
            for item in items {
                let res = analyze_expr(item, context);
                interpolate_errors(res, &mut errors);
            }
        }
        Expr::Hash(items) => {
            for (_key, value) in items {
                let res = analyze_expr(value, context);
                interpolate_errors(res, &mut errors);
            }
        }
        Expr::Function { body, parameters } => {
            let mut context = Context::new_function_block(context);
            // Make sure params are defined before checking the function
            for param in parameters {
                context.add(param.0.clone(), Expr::Symbol(param.0));
            }
            let res = analyze_stmts(body, Some(&mut context));
            interpolate_errors(res, &mut errors);
        }
        _ => {}
    };

    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(())
}

fn interpolate_errors(res: AnalysisResult, errors: &mut Vec<String>) {
    if let Err(errs) = res {
        for err in errs {
            errors.push(err)
        }
    }
}
