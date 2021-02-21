use std::collections::HashMap;

type SymbolScope = &'static str;

pub const GLOBAL_SCOPE: SymbolScope = "GLOBAL";
#[derive(Clone)]
pub struct Symbol {
  pub name: String,
  pub scope: SymbolScope,
  pub index: usize,
}

#[derive(Clone)]
pub struct SymbolTable {
  store: HashMap<String, Symbol>,
  num_definitions: usize
}

impl SymbolTable {
  pub fn new() -> SymbolTable {
    let store = HashMap::new();

    SymbolTable {
      store,
      num_definitions: 0
    }
  }

  pub fn define(&mut self, name: String) -> Symbol {
    let symbol = Symbol {
      name: name.clone(),
      index: self.num_definitions,
      scope: GLOBAL_SCOPE,
    };

    self.store.insert(name, symbol.clone());
    self.num_definitions += 1;

    symbol
  }

  pub fn resolve(&self, name: &str) -> Option<&Symbol> {
    self.store.get(name)
  }
}