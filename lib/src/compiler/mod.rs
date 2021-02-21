use std::{unreachable};
use num::FromPrimitive;

use ast::{Expr, Program, Stmt};
use code::{Instructions, Opcode};
use object::Object;
use symbol_table::{Symbol, SymbolTable};

use crate::ast;
use crate::code;
use crate::object;

pub mod symbol_table;

#[cfg(test)]
mod compiler_test;

#[derive(Clone)]
pub struct Compiler {
    instructions: Instructions,
    constants: Vec<Object>,
    // Last is the most recently emitted instructions
    // And previous is the one before that
    last_instruction: EmittedInstruction,
    previous_instruction: EmittedInstruction,
    pub symbols: SymbolTable
}

// Copy helps ALOT here
#[derive(Clone, Copy)]
pub struct EmittedInstruction {
    code: Opcode,
    position: usize
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
        let ins = EmittedInstruction {
            code: Opcode::Pop,
            position: 0
        };
        Compiler {
            instructions: Vec::new(),
            constants: Vec::new(),
            last_instruction: ins,
            previous_instruction: ins,
            symbols: SymbolTable::new()
        }
    }

    pub fn new_with_state(s: SymbolTable, constants: Vec<Object>) -> Self {
        let mut compiler = Self::new();
        compiler.symbols = s;
        compiler.constants = constants;

        compiler
    }

    pub fn compile(&mut self, program: Program) -> Result<(), String> {
        for stmt in program.0 {
            self.compile_stmt(stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Assign(name, value) => {
                self.compile_expr(value)?;

                let symbol = match name {
                    Expr::Ident(name) => {
                        self.symbols.define(name.0)
                    },
                    _ => unreachable!()
                };

                self.emit(Opcode::SetGlobal, vec![symbol.index]);

                Ok(())
            },
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
            Expr::Ident(ident) => {
                let symbol = self.symbols.resolve(&ident.0);
                match symbol {
                    Some(symbol) => {
                        self.emit(Opcode::GetGlobal, vec![symbol.index]);
                    },
                    None => return Err(format!("Undefined variable: {}", ident.0))
                }
                Ok(())
            },
            Expr::Infix(left, op, right) => {
                let op = op.as_str();
                if op == "<" || op == "<=" {
                    // Reverse the order, so we use less instructions
                    self.compile_expr(*right)?;
                    self.compile_expr(*left)?;

                    match op {
                        "<" => self.emit(Opcode::Greater, Vec::new()),
                        "<=" => self.emit(Opcode::GreaterEqual, Vec::new()),
                        _ => unreachable!()
                    };

                    return Ok(())
                }
                self.compile_expr(*left)?;
                self.compile_expr(*right)?;

                
                match op {
                    "+" => self.emit(Opcode::Add, vec![]),
                    "-" => self.emit(Opcode::Sub, Vec::new()),
                    "*" => self.emit(Opcode::Mul, Vec::new()),
                    "/" => self.emit(Opcode::Div, Vec::new()),
                    "%" => self.emit(Opcode::Mod, Vec::new()),
                    ">" => self.emit(Opcode::Greater, Vec::new()),
                    ">=" => self.emit(Opcode::GreaterEqual, Vec::new()),
                    "==" => self.emit(Opcode::Equal, Vec::new()),
                    "!=" => self.emit(Opcode::NotEqual, Vec::new()),
                    _ => return Err(format!("unknown operator {}", op)),
                };
                
                Ok(())
            }
            Expr::Number(num) => {
                let num = Object::Number(num);
                let constant = self.add_constant(num);
                self.emit(Opcode::Constant, vec![constant]);
                Ok(())
            },
            Expr::String(string) => {
                let string = Object::String(string);
                let constant = self.add_constant(string);
                self.emit(Opcode::Constant, vec![constant]);
                Ok(())
            },
            Expr::Boolean(b) => {
                let op = if b { Opcode::True } else { Opcode::False };
                self.emit(op, vec![]);
                Ok(())
            },
            Expr::Prefix(op, expr) => {
                self.compile_expr(*expr)?;

                match op.as_str() {
                    "!" => self.emit(Opcode::Bang, vec![]),
                    "-" => self.emit(Opcode::Minus, vec![]),
                    _ => return Err(format!("Unknown operator: {}", op))
                };
                Ok(())
            },
            Expr::If { condition, consequence, alternative} => {
                self.compile_expr(*condition)?;

                // Value will be changed later
                let jump_not_truthy_pos = 
                    self.emit(Opcode::JumpNotTruthy, vec![9999]);

                self.compile(consequence)?;

                if self.last_instruction_is(Opcode::Pop) {
                    self.remove_last_pop();
                }
                // Alternative Handling

                // Value will be changed later
                let jump_pos = 
                    self.emit(Opcode::Jump, vec![9999]);

                let after_consequence_pos = self.instructions.len();

                self.change_operand(jump_not_truthy_pos, after_consequence_pos);

                self.compile(alternative)?;

                if self.last_instruction_is(Opcode::Pop) {
                    self.remove_last_pop();
                }

                let after_alternative_pos = self.instructions.len();
                self.change_operand(jump_pos, after_alternative_pos);

                Ok(())
            },
            Expr::Array(elements) => {
                let len = elements.len();
                for el in elements {
                    self.compile_expr(el)?;
                }

                self.emit(Opcode::Array, vec![len]);
                Ok(())
            },
            _ => Ok(()),
        }
    }

    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn emit(&mut self, op: Opcode, operands: Vec<usize>) -> usize {
        let ins = code::make(op, operands);
        let pos = self.add_instructions(ins);

        self.set_last(op, pos);
        pos
    }

    fn add_instructions(&mut self, ins: Vec<u8>) -> usize {
        let old_pos = self.instructions.len();

        for i in ins {
            self.instructions.push(i);
        }

        old_pos
    }

    fn set_last(&mut self, op: Opcode, pos: usize) {
        let previous = self.last_instruction;
        let last = EmittedInstruction { code: op, position: pos };

        self.previous_instruction = previous;
        self.last_instruction = last;
    }

    fn last_instruction_is(&mut self, op: Opcode) -> bool {
        self.last_instruction.code == op
    }

    fn remove_last_pop(&mut self) {
        let pos = self.last_instruction.position;
        self.instructions = self.instructions[0..pos].to_vec();
        self.last_instruction = self.previous_instruction;
    }

    fn change_operand(&mut self, pos: usize, operand: usize) {
        let op = Opcode::from_u8(self.instructions[pos]).unwrap();
        let new = code::make(op, vec![operand]);

        self.replace_instruction(pos, new);
    }

    fn replace_instruction(&mut self, pos: usize, new: Instructions) {
        let mut i = 0;
        while i < new.len() {
            self.instructions[pos + i] = new[i];
            i += 1;
        }
    }
}
