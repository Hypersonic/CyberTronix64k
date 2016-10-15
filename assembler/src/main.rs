use std::env;
use std::process;
use std::collections::HashMap;
use std::fs::File;
use std::io::{stderr, Write, Read};

struct Register(u16);
struct Memory(u16);
struct Label(String);

enum Opcode {
  MoveImmediate(Register, u16),
  Move(Register, Memory),
  MoveDeref(Register, Memory),
  Exchange(Register, Memory),
  Add(Register, Memory),
  Sub(Register, Memory),
  And(Register, Memory),
  Or(Register, Memory),
  Xor(Register, Memory),
  ShiftRight(Register, Memory),
  ShiftLeft(Register, Memory),
  ShiftLogical(Register, Memory),
  JumpGreater(Register, Memory, Label),
  JumpLesser(Register, Memory, Label),
  JumpEqual(Register, Memory, Label),
  Halt,
}

struct Labels(HashMap<String, Label>);
struct Program(Vec<Opcode>);

macro_rules! eprint {
  ($($t:tt)*) => (
    write!(stderr(), $($t)*).expect("Writing to stderr failed")
  )
}
macro_rules! eprintln {
  ($($t:tt)*) => (
    writeln!(stderr(), $($t)*).expect("Writing to stderr failed")
  )
}

fn main() {
  let args = env::args_os().collect::<Vec<_>>();
  let filename = match args.get(1) {
    Some(f) => f,
    None => {
      eprintln!("usage: {} [filename]",
        args
          .get(0)
          .map(|own| own.to_string_lossy())
          .unwrap_or("program".into()),
      );
      process::exit(1);
    }
  };

  let mut file = match File::open(filename) {
    Ok(file) => file,
    Err(e) => {
      eprintln!(
        "Failed to open file: `{}'\nError: {}", filename.to_string_lossy(), e,
      );
      process::exit(1);
    }
  };

  let mut bytes = Vec::new();
  match file.read_to_end(&mut bytes) {
    Ok(_) => {},
    Err(e) => {
      eprintln!(
        "Failed to read file: `{}'\nError: {}", filename.to_string_lossy(), e,
      );
      process::exit(1);
    }
  }

  unsafe {
    println!("{}", String::from_utf8_unchecked(bytes));
  }
}
