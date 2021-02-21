use num::FromPrimitive;
use std::usize;

use code::{Instructions, Opcode};
use object::Object;

use crate::code;
use crate::compiler;
use crate::object;

#[cfg(test)]
mod vm_test;

const STACK_SIZE: usize = 2048;

pub struct VM {
    constants: Vec<Object>,
    instructions: Instructions,

    stack: Vec<Object>,
    sp: usize, // stack-point
}

impl VM {
    pub fn new(bytecode: compiler::Bytecode) -> Self {
        Self {
            instructions: bytecode.instructions,
            constants: bytecode.constants,

            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        }
    }

    pub fn stack_top(&self) -> Option<Object> {
        if self.sp == 0 {
            return None;
        }
        Some(self.stack[self.sp - 1].clone())
    }

    pub fn last_stack_top(&self) -> Option<&Object> {
        self.stack.get(self.sp)
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut ip = 0;
        while ip < self.instructions.len() {
            let op = Opcode::from_u8(self.instructions[ip]).unwrap();

            match op {
                Opcode::Constant => {
                    let ins = self.instructions[ip + 1..].to_vec();
                    let index = code::read_u16(ins.to_vec());

                    let constant = self.constants[index as usize].clone();
                    self.push(constant)?;

                    ip += 2;
                }
                Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod | Opcode::Greater | Opcode::GreaterEqual => {
                    self.execute_binary_op(op)?;
                },
                Opcode::Minus => self.execute_minus_op()?,
                Opcode::Bang => self.execute_bang_op()?,
                Opcode::Equal | Opcode::NotEqual => {
                    self.execute_equality_op(op)?;
                },
                Opcode::True => self.push(Object::Boolean(true))?,
                Opcode::False => self.push(Object::Boolean(false))?,
                Opcode::Pop => {
                    self.pop();
                },
                _ => {}
            };

            ip += 1;
        }

        Ok(())
    }

    fn execute_bang_op(&mut self) -> Result<(), String> {
        let operand = self.pop();

        let opposite = match operand {
            Object::Boolean(true) => Object::Boolean(false),
            Object::Boolean(false) => Object::Boolean(true),
            _ => Object::Boolean(false)
        };

        self.push(opposite)
    }

    fn execute_minus_op(&mut self) -> Result<(), String> {
        let operand = self.pop();

        let num = match operand {
            Object::Number(num) => num,
            num => return Err(format!("Can't negate type: {}", num))
        };

        let opposite = Object::Number(-num); 
        self.push(opposite)
    }

    fn execute_equality_op(&mut self, op: Opcode) -> Result<(), String> {
        let right = self.pop();
        let left = self.pop();

        let result = match op {
            Opcode::Equal => left == right,
            Opcode::NotEqual => left != right,
            _ => return Err(format!("Unsupported equality operator: {:?}", op))
        };

        self.push(Object::Boolean(result))
    }

    fn execute_binary_op(&mut self, op: Opcode) -> Result<(), String> {
        let right = self.pop();
        let left = self.pop();

        match (left, right) {
            (Object::Number(left), Object::Number(right)) => {
                self.execute_binary_number_op(op, left, right)
            }
            (left, right) => Err(format!(
                "Invalid types for binary operation: {} {}",
                left, right
            )),
        }
    }

    fn execute_binary_number_op(
        &mut self,
        op: Opcode,
        left: f64,
        right: f64,
    ) -> Result<(), String> {
        let res = match op {
            Opcode::Add => Object::Number(left + right),
            Opcode::Sub => Object::Number(left - right),
            Opcode::Mul => Object::Number(left * right),
            Opcode::Div => Object::Number(left / right),
            Opcode::Mod => Object::Number(left % right),

            Opcode::Greater => Object::Boolean(left > right),
            Opcode::GreaterEqual => Object::Boolean(left >= right),
            _ => return Err(format!("Unknown operator: {:?}", op)),
        };

        self.push(res)
    }

    pub fn push(&mut self, obj: Object) -> Result<(), String> {
        if self.sp >= STACK_SIZE {
            return Err("stack overflow".to_string());
        }
        if self.sp >= self.stack.len() {
            self.stack.push(obj);
        } else {
            self.stack[self.sp] = obj;
        }

        self.sp += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Object {
        self.sp -= 1;
        self.stack[self.sp].clone()
    }
}
