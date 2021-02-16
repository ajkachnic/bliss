use std::collections::HashMap;
use super::{EvalResult, object::{BuiltinFunc, Object}};

pub fn get_builtins() -> HashMap<String, Object> {
  let mut builtins = HashMap::new();

  // Array functions
  builtins.insert("head".to_string(), Object::Builtin(1, head));
  builtins.insert("init".to_string(), Object::Builtin(1, init));
  builtins.insert("last".to_string(), Object::Builtin(1, last));
  builtins.insert("tail".to_string(), Object::Builtin(1, tail));


  builtins.insert("len".to_string(), Object::Builtin(1, len));
  builtins.insert("log".to_string(), Object::Builtin(-1, log));

  return builtins;
}

fn len(args: Vec<Object>) -> EvalResult {
  let arg = args[0].clone();
  let len = match arg {
    Object::Array(arr) => arr.len(),
    Object::String(s) => s.len(),
    Object::Hash(hash) => hash.len(),
    Object::Function{ parameters, body: _, env: _} => parameters.len(),
    _ => 0
  };
  return Ok(Object::Number(len as f64));
}

fn log(args: Vec<Object>) -> EvalResult {
  for arg in args {
    println!("{}", arg);
  }
  return Ok(Object::Void);
}

fn init(args: Vec<Object>) -> EvalResult {
  if let Object::Array(array) = args[0].clone() {
    return Ok(Object::Array(array[0..array.len()-1].to_vec()));
  };
  return Err(format!("{} isn't an array", args[0]))
}
fn tail(args: Vec<Object>) -> EvalResult {
  if let Object::Array(array) = args[0].clone() {
    return Ok(Object::Array(array[1..array.len()].to_vec()));
  };
  return Err(format!("{} isn't an array", args[0]))
}

fn head(args: Vec<Object>) -> EvalResult {
  if let Object::Array(array) = args[0].clone() {
    return match array.first() {
      Some(val) => Ok(val.clone()),
      None => Ok(Object::Null)
    }
  }
  return Err(format!("{} isn't an array", args[0]))
}

fn last(args: Vec<Object>) -> EvalResult {
  if let Object::Array(array) = args[0].clone() {
    return match array.last() {
      Some(val) => Ok(val.clone()),
      None => Ok(Object::Null)
    }
  }
  return Err(format!("{} isn't an array", args[0]))
}