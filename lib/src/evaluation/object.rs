use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::{env::Environment, Evaluator};
use crate::ast::{BlockStatement, Ident};

pub type BuiltinFunc = fn(Vec<Object>, Rc<RefCell<Evaluator>>) -> Result<Object, String>;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Number(f64),
    String(String),
    Ident(Ident),
    Boolean(bool),
    Array(Vec<Object>),
    Hash(HashMap<Object, Object>),
    Return(Box<Object>),
    Function {
        parameters: Vec<Ident>,
        body: BlockStatement,
        env: Rc<RefCell<Environment>>,
    },
    Builtin(i32, BuiltinFunc),
    Void,
    Null,
}

impl Eq for Object {}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Number(value) => write!(f, "{}", value),
            Object::String(value) => write!(f, "{}", value),
            Object::Ident(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::Array(value) => {
                let items: Vec<String> = value.iter().map(|item| format!("{}", item)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Object::Return(_) => Ok(()),
            Object::Void => Ok(()),
            Object::Null => write!(f, "null"),
            Object::Hash(map) => {
                let items: Vec<String> = map
                    .iter()
                    .map(|(key, value)| format!("{} = {}", key, value))
                    .collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Object::Function {
                parameters,
                body,
                env: _,
            } => {
                let params: Vec<String> = parameters
                    .iter()
                    .map(|param| return param.clone().0)
                    .collect();
                write!(f, "fn ({}) -> {{\n{}\n}}", params.join(", "), body)
            }
            Object::Builtin(_, _) => write!(f, "[builtin func]"),
        }
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Object::Number(ref i) => {
                // TODO: Clean this up
                // As of now, if you put "0.1" and 0.1 into a hashmap, they would override one another. To avoid using unsafe code, this is best solution I could think of atm
                i.to_string().hash(state)
            }
            Object::Boolean(ref b) => b.hash(state),
            Object::String(ref s) => s.hash(state),
            _ => "".hash(state),
        }
    }
}

impl From<bool> for Object {
    fn from(value: bool) -> Object {
        return Object::Boolean(value);
    }
}
