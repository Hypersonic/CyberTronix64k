use std::fs::File;
use std::io::Write;

use parser::Opcode;

use clap::{Arg, App};

extern crate clap;
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
  fn new(filename: &str, print_labels: bool) -> Self {
    Program(parser::Parser::new(filename, print_labels).collect())
  }
}

fn main() {
  let matches =
    App::new("CT64k Assembler")
      .version("0.1")
      .author("Nicole Mazzuca <npmazzuca@gmail.com>")
      .about("A work in progress assembler for the CT64k")
      .arg(
        Arg::with_name("output")
          .short("o")
          .value_name("FILE")
          .help("Sets the output file to use")
          .required(true)
          .takes_value(true),
      ).arg(
        Arg::with_name("input")
          .help("Sets the input file to use")
          .required(true)
          .index(1),
      ).arg(
        Arg::with_name("print-labels")
          .short("p")
          .long("print-labels")
          .help("Sets whether the assembler prints the values of the labels")
      ).get_matches();

  let outfilename = matches.value_of("output").unwrap();
  let inpfilename = matches.value_of("input").unwrap();
  let print_labels = matches.is_present("print-labels");

  let program = Program::new(inpfilename, print_labels);
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
  match File::create(outfilename) {
    Ok(mut file) => match file.write_all(&out) {
      Ok(_) => {}
      Err(e) => error_np!("Error while writing: {}", e),
    },
    Err(e) => error_np!(
      "Failed to open output file: `{}'\nError: {}", outfilename, e,
    ),
  };
}
