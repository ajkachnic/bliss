use ast::{Expr, Program, Stmt};
use code::{Instructions, Opcode};
use object::Object;

use crate::ast;
use crate::code;
use crate::object;

#[cfg(test)]
mod compiler_test;

pub struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,
}

#[derive(Debug)]
pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Into<Bytecode> for Compiler {
    fn into(self) -> Bytecode {
        Bytecode {
            instructions: self.instructions,
            constants: self.constants,
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile(&mut self, program: Program) -> Result<(), String> {
        for stmt in program.0 {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {
                let res = self.compile_expr(expr);
                self.emit(Opcode::Pop, vec![]);

                res
            }
            _ => Ok(()),
        }
    }

    fn compile_expr(&mut self, expr: Expr) -> Result<(), String> {
        match expr {
            Expr::Infix(left, op, right) => {
                self.compile_expr(*left)?;
                self.compile_expr(*right)?;

                match op.as_str() {
                    "+" => self.emit(Opcode::Add, vec![]),
                    "-" => self.emit(Opcode::Sub, Vec::new()),
                    "*" => self.emit(Opcode::Mul, Vec::new()),
                    "/" => self.emit(Opcode::Div, Vec::new()),
                    "%" => self.emit(Opcode::Mod, Vec::new()),
                    _ => return Err(format!("unknown operator {}", op)),
                };


                Ok(())
            }
            Expr::Number(num) => {
                let num = Object::Number(num);
                let constant = self.add_constant(num);
                self.emit(Opcode::Constant, vec![constant]);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn emit(&mut self, op: Opcode, operands: Vec<usize>) -> usize {
        let ins = code::make(op, operands);
        self.add_instructions(ins)
    }

    fn add_instructions(&mut self, ins: Vec<u8>) -> usize {
        let old_pos = self.instructions.len();

        for i in ins {
            self.instructions.push(i);
        }

        old_pos
    }
}
