use std::collections;

use crate::errors::RuntimeError;
use crate::expression::Value;
use crate::token::Token;

pub struct Environment {
  values: collections::HashMap<String, Value>,
}

impl Environment {
  pub fn new() -> Self {
    Self {
      values: collections::HashMap::new(),
    }
  }

  pub fn define(&mut self, name: String, value: Value) {
    self.values.insert(name, value);
  }

  pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
    let identifier = name.identifier();

    self
      .values
      .get(&identifier)
      .map(|v| v.clone())
      .ok_or_else(|| {
        RuntimeError::from_token(name, format!("Undefined variable '{}'.", identifier))
      })
  }
}
