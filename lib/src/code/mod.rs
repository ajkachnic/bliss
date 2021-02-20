use bytes::{BufMut, BytesMut};
use num::FromPrimitive;
use std::collections::HashMap;

#[cfg(test)]
#[path = "./code_test.rs"]
mod code_test;

pub type Instructions = Vec<u8>;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Opcode {
    Constant = 0,
    Add,
    Pop,
    Sub,
    Mul,
    Div,
    Mod,
    True,
    False,
}

impl FromPrimitive for Opcode {
    fn from_u64(op: u64) -> Option<Self> {
        Self::from_i64(op as i64)
    }

    fn from_i64(op: i64) -> Option<Self> {
        match op {
            0 => Some(Opcode::Constant),
            1 => Some(Opcode::Add),
            2 => Some(Opcode::Pop),
            3 => Some(Opcode::Sub),
            4 => Some(Opcode::Mul),
            5 => Some(Opcode::Div),
            6 => Some(Opcode::Mod),
            7 => Some(Opcode::True),
            8 => Some(Opcode::False),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Definition {
    pub name: String,
    pub operand_widths: Vec<u8>,
}

fn get_definitions() -> HashMap<Opcode, Definition> {
    let mut defs = HashMap::new();

    defs.insert(
        Opcode::Constant,
        Definition {
            name: "OpConstant".to_string(),
            operand_widths: vec![2],
        },
    );

    defs.insert(
        Opcode::Add,
        Definition {
            name: "OpAdd".to_string(),
            operand_widths: vec![],
        },
    );
    defs.insert(
        Opcode::Sub,
        Definition {
            name: "OpSub".to_string(),
            operand_widths: vec![],
        },
    );
    defs.insert(
        Opcode::Mul,
        Definition {
            name: "OpMul".to_string(),
            operand_widths: vec![],
        },
    );
    defs.insert(
        Opcode::Div,
        Definition {
            name: "OpDiv".to_string(),
            operand_widths: vec![],
        },
    );
    defs.insert(
        Opcode::Mod,
        Definition {
            name: "OpMod".to_string(),
            operand_widths: vec![],
        },
    );

    defs.insert(
        Opcode::Pop,
        Definition {
            name: "OpPop".to_string(),
            operand_widths: vec![],
        },
    );

    defs
}

pub fn lookup(op: u8) -> Result<Definition, String> {
    let defs = get_definitions();

    let code = match Opcode::from_u8(op) {
        Some(code) => code,
        None => return Err(format!("Unknown opcode {}", op)),
    };
    if let Some(def) = defs.get(&code) {
        return Ok(def.clone());
    }
    Err(format!("No definition found for Opcode {:?}", code))
}

pub fn make(op: Opcode, operands: Vec<usize>) -> Instructions {
    let definitions = get_definitions();

    let def: Definition = match definitions.get(&op) {
        Some(def) => def.clone(),
        None => return Vec::new(),
    };

    let mut instruction_len = 1;
    for w in &def.operand_widths {
        instruction_len += w;
    }

    let mut instructions = BytesMut::with_capacity(instruction_len as usize);

    instructions.put_u8(op as u8);

    for (operand, width) in operands.iter().zip(def.operand_widths) {
        match width {
            2 => instructions.put_u16(*operand as u16),
            _ => {}
        }
    }
    instructions.to_vec()
}
pub fn pretty(ins: Instructions) -> String {
    let mut out = String::new();

    let mut i = 0;
    while i < ins.len() {
        let def = match lookup(ins[i] as u8) {
            Ok(def) => def,
            Err(err) => {
                out.push_str(&format!("ERROR: {}\n", err));
                continue;
            }
        };

        let (operands, read) = read_operands(def.clone(), ins[i + 1..].to_vec());

        out.push_str(&format!("{:04} {}\n", i, fmt_instruction(def, operands)));
        i += 1 + read
    }

    out
}

pub fn fmt_instruction(def: Definition, operands: Vec<isize>) -> String {
    let count = def.operand_widths.len();

    match count {
        0 => def.name,
        1 => format!("{} {}", def.name, operands[0]),
        _ => format!("ERROR: unhandled operandCount for {}\n", def.name),
    }
}

pub fn read_operands(def: Definition, ins: Instructions) -> (Vec<isize>, usize) {
    let mut operands = Vec::with_capacity(def.operand_widths.len());
    let mut offset = 0;

    for (index, width) in def.operand_widths.iter().enumerate() {
        match width {
            2 => operands.push(read_u16(ins[offset..ins.len()].to_vec()) as isize),
            _ => {}
        }
        offset += *width as usize
    }

    (operands, offset)
}

pub fn read_u16(ins: Instructions) -> u16 {
    let s: [u8; 2] = [ins[0], ins[1]];

    u16::from_be_bytes(s)
}
