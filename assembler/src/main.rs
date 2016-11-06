use std::env;
use std::fs::File;
use std::io::Write;

use parser::Opcode;

#[macro_use]
extern crate maplit;

#[macro_use]
mod macros;
mod lexer;
mod parser;

pub enum Instruction {
  Arith {
    op: Opcode,
    reg: u16,
    imm: u16,
  },
  Jump {
    op: Opcode,
    reg: u16,
    imm: u16,
    label: u16,
  },
  Data(Vec<u16>),
  ByteData(Vec<u8>),
}

struct Program(Vec<Instruction>);

impl Program {
  fn new(filename: &str) -> Self {
    Program(parser::Parser::new(filename).collect())
  }
}

fn main() {
  let args = env::args_os().collect::<Vec<_>>();
  let inpfilename = match args.get(1) {
    Some(f) => match f.to_str() {
      Some(s) => s,
      None => error_np!("Filename must be valid utf-8; is {:?}", f),
    },
    None => error_np!(
      "usage: {} filename -o output",
      args.get(0).map(|own| own.to_string_lossy()).unwrap_or("program".into()),
    ),
  };
  match args.get(2) {
    Some(s) if s == "-o" => {},
    Some(s) => {
      error_np!(
        "The second argument *must* be `-o' until I fix argument handling\n\
        (it was {})",
        s.to_string_lossy(),
      );
    }
    None => error_np!(
      "usage: {} filename -o output",
      args.get(0).map(|own| own.to_string_lossy()).unwrap_or("program".into()),
    ),
  }
  let outfilename = match args.get(3) {
    Some(f) => match f.to_str() {
      Some(s) => s,
      None => error_np!("Filename must be valid utf-8; is {:?}", f),
    },
    None => error_np!(
      "usage: {} filename -o output",
      args.get(0).map(|own| own.to_string_lossy()).unwrap_or("program".into()),
    ),
  };

  let program = Program::new(&inpfilename);
  let mut out = Vec::new();
  for op in program.0 {
    use Instruction::*;

    fn write(out: &mut Vec<u8>, to_write: &[u16]) {
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
          error_np!("Error while writing: {}", e);
        }
      }
    };

    match op {
      Arith {
        op,
        reg,
        imm,
      } => write(&mut out, &[(op.to_u16() << 10) | reg, imm]),
      Jump {
        op,
        reg,
        imm,
        label,
      } => write(&mut out, &[(op.to_u16() << 10) | reg, imm, label]),
      Data(nums) => write(&mut out, &nums),
      ByteData(nums) => if nums.len() % 2 == 0 {
        match out.write_all(&nums) {
          Ok(_) => {}
          Err(_) => unreachable!(),
        }
      } else {
        unreachable!()
      },
    }
  }
  match File::create(&outfilename) {
    Ok(mut file) => match file.write_all(&out) {
      Ok(_) => {}
      Err(e) => error_np!("Error while writing: {}", e),
    },
    Err(e) => error_np!(
      "Failed to open output file: `{}'\nError: {}", outfilename, e,
    ),
  };
}
