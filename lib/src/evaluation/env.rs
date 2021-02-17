use super::object::Object;
use std::collections::HashMap;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(PartialEq, Clone, Debug)]
pub struct Environment {
    store: HashMap<String, Object>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            store: HashMap::new(),
            parent: None,
        }
    }

    pub fn get_store(&self) -> &HashMap<String, Object> {
        &self.store
    }

    pub fn new_enclosed(parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            store: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn get(&self, key: String) -> Option<Object> {
        match self.store.get(&key) {
            Some(value) => Some(value.clone()),
            None => match self.parent {
                Some(ref parent) => parent.borrow_mut().get(key),
                None => None,
            },
        }
    }

    pub fn set(&mut self, key: String, value: Object) {
        self.store.insert(key, value);
    }

    pub fn has(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }
}
