use std::usize;
use num::FromPrimitive;

use code::{Instructions, Opcode};
use object::Object;

use crate::code;
use crate::object;
use crate::compiler;

const STACK_SIZE: usize = 2048;


pub struct VM {
  constants: Vec<Object>,
  instructions: Instructions,

  stack: Vec<Object>,
  sp: usize // stack-point
}

impl VM {
  pub fn new(bytecode: compiler::Bytecode) -> Self {
    Self {
      instructions: bytecode.instructions,
      constants: bytecode.constants,

      stack: Vec::with_capacity(STACK_SIZE),
      sp: 0
    }
  }

  pub fn stack_top(&self) -> Option<Object> {
    if self.sp == 0 {
      return None
    }
    Some(self.stack[self.sp - 1].clone())
  }

  pub fn run(&mut self) -> Result<(), String> {
    let mut ip = 0;
    while ip < self.instructions.len() {
      let op = Opcode::from_u8(self.instructions[ip]);

      match op {};

      ip += 1;
    }

    Ok(())
  }
}