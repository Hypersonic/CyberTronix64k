#![allow(unused)]

use std::env;
use std::process;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Read};
use std::fmt::{self, Debug, Display};

macro_rules! eprint {
  ($($t:tt)*) => ({
    use std::io::Write;
    write!(::std::io::stderr(), $($t)*).expect("Writing to stderr failed")
  })
}
macro_rules! eprintln {
  ($($t:tt)*) => ({
    use std::io::Write;
    writeln!(::std::io::stderr(), $($t)*).expect("Writing to stderr failed")
  })
}

mod parser;

pub enum Number {
  Immediate(u16),
  Label(String),
}

impl Display for Number {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Number::Immediate(i) => write!(f, "0x{:x}", i),
      Number::Label(ref s) => write!(f, "{}", s),
    }
  }
}

pub struct Opcode {
  variant: OpcodeVariant,
  reg: Number,
  mem: Number,
}

pub enum OpcodeVariant {
  MoveImmediate,
  Move,
  MoveDeref,
  Load,
  Store,
  Add,
  Sub,
  And,
  Or,
  Xor,
  ShiftRight,
  ShiftLeft,
  ShiftArithmetic,
  JumpGreater(Number),
  JumpLesser(Number),
  JumpEqual(Number),
}

impl Opcode {
  fn size(&self) -> u16 {
    use OpcodeVariant::*;
    match self.variant {
      MoveImmediate | Move | MoveDeref | Load | Store | Add | Sub
      | And | Or | Xor | ShiftRight | ShiftLeft | ShiftArithmetic => {
        2
      }
      JumpGreater(..) | JumpLesser(..) | JumpEqual(..) => {
        3
      }
    }
  }
}

impl Debug for Opcode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use OpcodeVariant::*;
    match self.variant {
      MoveImmediate => write!(f, "MI {}, {}", self.reg, self.mem),
      Move => write!(f, "MV {}, {}", self.reg, self.mem),
      MoveDeref => write!(f, "MD {}, {}", self.reg, self.mem),
      Load => write!(f, "LD {}, {}", self.reg, self.mem),
      Store => write!(f, "ST {}, {}", self.reg, self.mem),
      Add => write!(f, "AD {}, {}", self.reg, self.mem),
      Sub => write!(f, "SB {}, {}", self.reg, self.mem),
      And => write!(f, "ND {}, {}", self.reg, self.mem),
      Or => write!(f, "OR {}, {}", self.reg, self.mem),
      Xor => write!(f, "XR {}, {}", self.reg, self.mem),
      ShiftRight => write!(f, "SR {}, {}", self.reg, self.mem),
      ShiftLeft => write!(f, "SL {}, {}", self.reg, self.mem),
      ShiftArithmetic => write!(f, "SA {}, {}", self.reg, self.mem),
      JumpGreater(ref label) =>
        write!(f, "JG {}, {}, {}", self.reg, self.mem, label),
      JumpLesser(ref label) =>
        write!(f, "JL {}, {}, {}", self.reg, self.mem, label),
      JumpEqual(ref label) =>
        write!(f, "JQ {}, {}, {}", self.reg, self.mem, label),
    }
  }
}

struct Program {
  labels: HashMap<String, u16>,
  program: Vec<(Opcode, u16)>,
}

impl Program {
  fn new(input: Vec<u8>) -> Self {
    let mut this = Program {
      labels: HashMap::new(),
      program: Vec::new(),
    };

    let mut offset = 0x1000;
    for op_or_label in parser::Parser::new(input) {
      match op_or_label {
        parser::OpOrLabel::Op(op) => {
          let tmp = op.size();
          this.program.push((op, offset));
          offset += tmp;
        }
        parser::OpOrLabel::Label(label, idx) => {
          match this.labels.insert(label, idx + offset) {
            Some(old) => {
              eprintln!("Attempted to duplicate label definition: {}", old);
              process::exit(1);
            }
            None => {}
          }
        }
      }
    }

    this
  }
}

fn main() {
  let args = env::args_os().collect::<Vec<_>>();
  let inpfilename = match args.get(1) {
    Some(f) => f,
    None => {
      eprintln!("usage: {} filename -o output",
        args
          .get(0)
          .map(|own| own.to_string_lossy())
          .unwrap_or("program".into()),
      );
      process::exit(1);
    }
  };
  let outfilename = match args.get(3) {
    Some(f) => f,
    None => {
      eprintln!("usage: {} filename -o output",
        args
          .get(0)
          .map(|own| own.to_string_lossy())
          .unwrap_or("program".into()),
      );
      process::exit(1);
    }
  };

  let mut inp = match File::open(inpfilename) {
    Ok(file) => file,
    Err(e) => {
      eprintln!(
        "Failed to open input file: `{}'\nError: {}",
        inpfilename.to_string_lossy(),
        e,
      );
      process::exit(1);
    }
  };
  let mut out = match File::create(outfilename) {
    Ok(file) => file,
    Err(e) => {
      eprintln!(
        "Failed to open output file: `{}'\nError: {}",
        outfilename.to_string_lossy(),
        e,
      );
      process::exit(1);
    }
  };

  let mut bytes = Vec::new();
  match inp.read_to_end(&mut bytes) {
    Ok(_) => {},
    Err(e) => {
      eprintln!(
        "Failed to read file: `{}'\nError: {}",
        inpfilename.to_string_lossy(),
        e,
      );
      process::exit(1);
    }
  }

  let program = Program::new(bytes);
  let labels = program.labels;
  for op in program.program {
    use OpcodeVariant::*;
    let offset = op.1;

    fn write(out: &mut File, to_write: &[u16]) {
      let res = out.write_all(
        unsafe {
          std::slice::from_raw_parts(
            to_write.as_ptr() as *const u8,
            to_write.len() * 2,
          )
        }
      );
      match res {
        Ok(_) => {}
        Err(e) => {
          eprintln!("Error while writing: {}", e);
          process::exit(1);
        }
      }
    };
    let get_number = |num: &Number, labels: &HashMap<String, u16>| -> u16 {
      match *num {
        Number::Immediate(n) => n,
        Number::Label(ref s) => {
          if s.is_empty() {
            offset
          } else if let Some(&n) = labels.get(s) {
            n
          } else {
            eprintln!("Label not found: {}", s);
            process::exit(1);
          }
        }
      }
    };
    let mut arith = |out: &mut File, opcode: u16, reg: &Number, mem: &Number| {
      let regn = get_number(&reg, &labels);
      if regn >= 0x1000 {
        eprintln!("Register label out of range: {} is 0x{:x}", reg, regn);
        process::exit(1);
      }
      let mem = get_number(&mem, &labels);
      write(out, &[(opcode << 12) | regn, mem]);
    };
    let mut jump = |
      out: &mut File, opcode: u16, reg: &Number, mem: &Number, label: &Number
    | {
      let regn = get_number(&reg, &labels);
      if regn >= 0x1000 {
        eprintln!("Register label out of range: {} is 0x{:x}", reg, regn);
        process::exit(1);
      }
      let mem = get_number(&mem, &labels);
      let label = get_number(&label, &labels);
      write(out, &[(opcode << 12) | regn, mem, label]);
    };

    match (op.0).variant {
      MoveImmediate => arith(&mut out, 0x0, &(op.0).reg, &(op.0).mem),
      Move => arith(&mut out, 0x1, &(op.0).reg, &(op.0).mem),
      MoveDeref => arith(&mut out, 0x2, &(op.0).reg, &(op.0).mem),
      Load => arith(&mut out, 0x3, &(op.0).reg, &(op.0).mem),
      Store => arith(&mut out, 0x4, &(op.0).reg, &(op.0).mem),
      Add => arith(&mut out, 0x5, &(op.0).reg, &(op.0).mem),
      Sub => arith(&mut out, 0x6, &(op.0).reg, &(op.0).mem),
      And => arith(&mut out, 0x7, &(op.0).reg, &(op.0).mem),
      Or => arith(&mut out, 0x8, &(op.0).reg, &(op.0).mem),
      Xor => arith(&mut out, 0x9, &(op.0).reg, &(op.0).mem),
      ShiftRight => arith(&mut out, 0xA, &(op.0).reg, &(op.0).mem),
      ShiftLeft => arith(&mut out, 0xB, &(op.0).reg, &(op.0).mem),
      ShiftArithmetic => arith(&mut out, 0xC, &(op.0).reg, &(op.0).mem),
      JumpGreater(ref label) =>
        jump(&mut out, 0xD, &(op.0).reg, &(op.0).mem, label),
      JumpLesser(ref label) =>
        jump(&mut out, 0xE, &(op.0).reg, &(op.0).mem, label),
      JumpEqual(ref label) =>
        jump(&mut out, 0xF, &(op.0).reg, &(op.0).mem, label),
    }
  }
}
