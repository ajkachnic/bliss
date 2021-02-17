use std::{collections::HashMap, default::Default};

use crate::ast::Expr;

#[derive(Clone, Debug)]
pub struct Context {
    // Parent context
    pub parent: Option<Box<Context>>,
    pub in_function: bool,
    // All the declared variables in a context
    pub locals: HashMap<String, Expr>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            parent: None,
            in_function: false,
            locals: HashMap::new(),
        }
    }
}

impl Context {
    pub fn new_child_block(&self) -> Context {
        Context {
            parent: Some(Box::new(self.clone())),
            in_function: self.in_function,
            ..Default::default()
        }
    }
    pub fn new_function_block(&self) -> Context {
        Context {
            parent: Some(Box::new(self.clone())),
            in_function: true,
            ..Default::default()
        }
    }

    pub fn add(&mut self, name: String, expr: Expr) {
        // Add a new item to the vec
        self.locals.insert(name, expr);
    }

    // Returns whether or not a value exists in our identifiers list
    pub fn has(&self, name: String) -> bool {
        if self.locals.contains_key(&name) {
            return true;
        }
        let mut found = false;
        while let Some(parent) = self.parent.clone() {
            found = parent.locals.contains_key(&name);
            if found {
                break;
            }
        }
        found
    }
    // Returns a possible value
    pub fn lookup(&self, name: String) -> Option<Expr> {
        if let Some(value) = self.locals.get(&name) {
            return Some(value.clone());
        };
        if let Some(parent) = self.parent.clone() {
            return parent.lookup(name);
        }
        None
    }
}
