use std::{collections::HashMap, rc::Rc};

use std::cell::RefCell;

use super::{object::Object, EvalResult, Evaluator};

pub fn get_builtins<'a>() -> HashMap<String, Object<'a>> {
    let mut builtins = HashMap::new();

    // Array functions
    builtins.insert("head".to_string(), Object::Builtin(1, head));
    builtins.insert("init".to_string(), Object::Builtin(1, init));
    builtins.insert("last".to_string(), Object::Builtin(1, last));
    builtins.insert("tail".to_string(), Object::Builtin(1, tail));

    builtins.insert("len".to_string(), Object::Builtin(1, len));
    builtins.insert("log".to_string(), Object::Builtin(-1, log));

    builtins.insert("map".to_string(), Object::Builtin(2, map));

    builtins
}

fn len<'a>(args: Vec<Object<'a>>, _: Rc<RefCell<Evaluator>>) -> EvalResult<'a> {
    let arg = args[0].clone();
    let len = match arg {
        Object::Array(arr) => arr.len(),
        Object::String(s) => s.len(),
        Object::Hash(hash) => hash.len(),
        Object::Function {
            parameters,
            body: _,
            env: _,
        } => parameters.len(),
        _ => 0,
    };
    Ok(Object::Number(len as f64))
}

fn log<'a>(args: Vec<Object>, _: Rc<RefCell<Evaluator>>) -> EvalResult<'a> {
    for arg in args {
        println!("{}", arg);
    }
    Ok(Object::Void)
}

fn init<'a>(args: Vec<Object<'a>>, _: Rc<RefCell<Evaluator>>) -> EvalResult<'a> {
    if let Object::Array(array) = args[0].clone() {
        return Ok(Object::Array(array[0..array.len() - 1].to_vec()));
    };
    Err(format!("{} isn't an array", args[0]))
}
fn tail<'a>(args: Vec<Object<'a>>, _: Rc<RefCell<Evaluator>>) -> EvalResult<'a> {
    if let Object::Array(array) = args[0].clone() {
        return Ok(Object::Array(array[1..array.len()].to_vec()));
    };
    Err(format!("{} isn't an array", args[0]))
}

fn head<'a>(args: Vec<Object<'a>>, _: Rc<RefCell<Evaluator>>) -> EvalResult<'a> {
    if let Object::Array(array) = args[0].clone() {
        return match array.first() {
            Some(val) => Ok(val.clone()),
            None => Ok(Object::Null),
        };
    }
    Err(format!("{} isn't an array", args[0]))
}

fn last<'a>(args: Vec<Object<'a>>, _: Rc<RefCell<Evaluator>>) -> EvalResult<'a> {
    if let Object::Array(array) = args[0].clone() {
        return match array.last() {
            Some(val) => Ok(val.clone()),
            None => Ok(Object::Null),
        };
    }
    Err(format!("{} isn't an array", args[0]))
}

fn map<'a>(args: Vec<Object<'a>>, eval: Rc<RefCell<Evaluator<'a>>>) -> EvalResult<'a> {
    if let Object::Array(array) = args[0].clone() {
        match args[1].clone() {
            Object::Function {
                body,
                env,
                parameters,
            } => {
                let function = Object::Function {
                    body,
                    env,
                    parameters,
                };
                let mut arr = vec![];
                for element in array {
                    let res = eval
                        .borrow_mut()
                        .eval_function_call(function.clone(), vec![element])?;
                    arr.push(res);
                }
                return Ok(Object::Array(arr));
            }
            arg => return Err(format!("Expected function, got {}", arg)),
        }
    }
    Err(format!("{} isn't an array", args[0]))
}
