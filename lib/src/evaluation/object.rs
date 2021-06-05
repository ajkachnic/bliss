use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::{env::Environment, Evaluator};
use crate::ast::{BlockStatement, Ident};

pub type BuiltinFunc<'a> =
    fn(Vec<Object<'a>>, Rc<RefCell<Evaluator<'a>>>) -> Result<Object<'a>, String>;

#[derive(Debug, Clone, PartialEq)]
pub enum Object<'a> {
    Number(f64),
    String(String),
    Symbol(String),
    Ident(Ident),
    Boolean(bool),
    Array(Vec<Object<'a>>),
    Hash(HashMap<String, Object<'a>>),
    Return(Box<Object<'a>>),
    Function {
        parameters: Vec<Ident>,
        body: BlockStatement,
        env: Rc<RefCell<Environment<'a>>>,
    },
    Builtin(isize, BuiltinFunc<'a>),
    Void,
    Null,
}

// impl Eq for Object {}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Number(value) => write!(f, "{}", value),
            Object::String(value) => write!(f, "'{}'", value),
            Object::Symbol(value) => write!(f, ":{}", value),
            Object::Ident(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::Array(value) => {
                let items: Vec<String> = value.iter().map(|item| format!("{}", item)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Object::Return(_) => Ok(()),
            Object::Void => write!(f, "<void>"),
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
                let params: Vec<String> = parameters.iter().map(|param| param.clone().0).collect();
                write!(f, "fn ({}) -> {{\n{}\n}}", params.join(", "), body)
            }
            Object::Builtin(_, _) => write!(f, "[builtin func]"),
        }
    }
}

impl<'a> From<bool> for Object<'a> {
    fn from(value: bool) -> Self {
        Object::Boolean(value)
    }
}
