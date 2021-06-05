use super::object::Object;
use std::collections::HashMap;

use std::cell::RefCell;
use std::rc::Rc;

type Env<'a> = Rc<RefCell<Environment<'a>>>;

#[derive(PartialEq, Clone, Debug)]
pub struct Environment<'a> {
    store: HashMap<String, Object<'a>>,
    parent: Option<Env<'a>>,
}

impl<'a> Default for Environment<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Environment<'a> {
    pub fn new() -> Environment<'a> {
        Environment {
            store: HashMap::new(),
            parent: None,
        }
    }

    pub fn get_store(&self) -> &HashMap<String, Object<'a>> {
        &self.store
    }

    pub fn new_enclosed(parent: &Env<'a>) -> Self {
        Environment {
            store: HashMap::new(),
            parent: Some(parent.clone()),
        }
    }

    pub fn get(&self, key: String) -> Option<Object<'a>> {
        match self.store.get(&key) {
            Some(value) => Some(value.clone()),
            None => match self.parent {
                Some(ref parent) => parent.borrow().get(key),
                None => None,
            },
        }
    }

    pub fn set(&mut self, key: String, value: Object<'a>) {
        self.store.insert(key, value);
    }

    pub fn has(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }
}

impl<'a> From<Environment<'a>> for Rc<RefCell<Environment<'a>>> {
    fn from(val: Environment<'a>) -> Self {
        Rc::new(RefCell::new(val))
    }
}
