#![allow(unused)]

use std::env;
use std::process;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Read};
use std::fmt::{self, Debug};

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

pub struct Register(u16);
pub struct Memory(u16);
pub struct Label(String);

pub enum Opcode {
  MoveImmediate(Register, u16),
  Move(Register, Memory),
  MoveDeref(Register, Memory),
  Load(Register, Memory),
  Store(Register, Memory),
  Add(Register, Memory),
  Sub(Register, Memory),
  And(Register, Memory),
  Or(Register, Memory),
  Xor(Register, Memory),
  ShiftRight(Register, Memory),
  ShiftLeft(Register, Memory),
  ShiftArithmetic(Register, Memory),
  JumpGreater(Register, Memory, Label),
  JumpLesser(Register, Memory, Label),
  JumpEqual(Register, Memory, Label),
}

impl Opcode {
  fn size(&self) -> u16 {
    use Opcode::*;
    match *self {
      MoveImmediate(..) | Move(..) | MoveDeref(..) | Load(..) | Store(..)
      | Add(..) | Sub(..) | And(..) | Or(..) | Xor(..)
      | ShiftRight(..) | ShiftLeft(..) | ShiftArithmetic(..) => {
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
    use Opcode::*;
    match *self {
      MoveImmediate(ref reg, ref imm) =>
        write!(f, "MI 0x{:x}, 0x{:x}", reg.0, imm),
      Move(ref reg, ref mem) =>
        write!(f, "MV 0x{:x}, 0x{:x}", reg.0, mem.0),
      MoveDeref(ref reg, ref mem) =>
        write!(f, "MD 0x{:x}, 0x{:x}", reg.0, mem.0),
      Load(ref reg, ref mem) =>
        write!(f, "LD 0x{:x}, 0x{:x}", reg.0, mem.0),
      Store(ref reg, ref mem) =>
        write!(f, "ST 0x{:x}, 0x{:x}", reg.0, mem.0),
      Add(ref reg, ref mem) =>
        write!(f, "AD 0x{:x}, 0x{:x}", reg.0, mem.0),
      Sub(ref reg, ref mem) =>
        write!(f, "SB 0x{:x}, 0x{:x}", reg.0, mem.0),
      And(ref reg, ref mem) =>
        write!(f, "ND 0x{:x}, 0x{:x}", reg.0, mem.0),
      Or(ref reg, ref mem) =>
        write!(f, "OR 0x{:x}, 0x{:x}", reg.0, mem.0),
      Xor(ref reg, ref mem) =>
        write!(f, "XR 0x{:x}, 0x{:x}", reg.0, mem.0),
      ShiftRight(ref reg, ref mem) =>
        write!(f, "SR 0x{:x}, 0x{:x}", reg.0, mem.0),
      ShiftLeft(ref reg, ref mem) =>
        write!(f, "SL 0x{:x}, 0x{:x}", reg.0, mem.0),
      ShiftArithmetic(ref reg, ref mem) =>
        write!(f, "SA 0x{:x}, 0x{:x}", reg.0, mem.0),
      JumpGreater(ref reg, ref mem, ref label) =>
        write!(f, "JG 0x{:x}, 0x{:x}, {}", reg.0, mem.0, label.0),
      JumpLesser(ref reg, ref mem, ref label) =>
        write!(f, "JL 0x{:x}, 0x{:x}, {}", reg.0, mem.0, label.0),
      JumpEqual(ref reg, ref mem, ref label) =>
        write!(f, "JQ 0x{:x}, 0x{:x}, {}", reg.0, mem.0, label.0),
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

    let mut offset = 0;
    for op_or_label in parser::Parser::new(input) {
      match op_or_label {
        parser::OpOrLabel::Op(op) => {
          let tmp = op.size();
          this.program.push((op, offset));
          offset += tmp;
        }
        parser::OpOrLabel::Label(label, idx) => {
          this.labels.insert(label, idx);
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
    use Opcode::*;
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
    let mut jump = |
      out: &mut File, lge: i8, reg: Register, mem: Memory, label: Label
    | {
      let label = match labels.get(&label.0) {
        Some(&l) => l,
        None => {
          if (label.0).is_empty() {
            offset
          } else {
            eprintln!("Label not found: {}", label.0);
            process::exit(1);
          }
        }
      };

      let jr = label.wrapping_sub(offset + 3);

      if lge > 0 {
        write(out, &[(0xD << 12) | reg.0, mem.0, jr]);
      } else if lge < 0 {
        write(out, &[(0xE << 12) | reg.0, mem.0, jr]);
      } else {
        write(out, &[(0xF << 12) | reg.0, mem.0, jr]);
      }
    };

    match op.0 {
      MoveImmediate(reg, imm) => write(&mut out, &[(0x0 << 12) | reg.0, imm]),
      Move(reg, mem) => write(&mut out, &[(0x1 << 12) | reg.0, mem.0]),
      MoveDeref(reg, mem) => write(&mut out, &[(0x2 << 12) | reg.0, mem.0]),
      Load(reg, mem) => write(&mut out, &[(0x3 << 12) | reg.0, mem.0]),
      Store(reg, mem) => write(&mut out, &[(0x4 << 12) | reg.0, mem.0]),
      Add(reg, mem) => write(&mut out, &[(0x5 << 12) | reg.0, mem.0]),
      Sub(reg, mem) => write(&mut out, &[(0x6 << 12) | reg.0, mem.0]),
      And(reg, mem) => write(&mut out, &[(0x7 << 12) | reg.0, mem.0]),
      Or(reg, mem) => write(&mut out, &[(0x8 << 12) | reg.0, mem.0]),
      Xor(reg, mem) => write(&mut out, &[(0x9 << 12) | reg.0, mem.0]),
      ShiftRight(reg, mem) => write(&mut out, &[(0xA << 12) | reg.0, mem.0]),
      ShiftLeft(reg, mem) => write(&mut out, &[(0xB << 12) | reg.0, mem.0]),
      ShiftArithmetic(reg, mem) =>
        write(&mut out, &[(0xC << 12) | reg.0, mem.0]),
      JumpGreater(reg, mem, label) => jump(&mut out, 1, reg, mem, label),
      JumpLesser(reg, mem, label) => jump(&mut out, -1, reg, mem, label),
      JumpEqual(reg, mem, label) => jump(&mut out, 0, reg, mem, label),
    }
  }
}
