use std::collections::HashMap;
use std::path::{Path, PathBuf};
use Instruction;

use lexer::{
  Directive, DirectiveVar, Lexer, OpArg, Position, Public,
};

const CODE_START: u16 = 0x400;

const REG_IP: u16 = 0x0;
const REG_SP: u16 = 0x2;
const REG_BP: u16 = 0x4;
const REG_SC0: u16 = 0x6;
const REG_SC1: u16 = 0x8;
const REG_SC2: u16 = 0xA;
const REG_SC3: u16 = 0xC;

#[derive(Copy, Clone)]
enum BaseOp {
  Mvi = 0x0,
  Mv = 0x1,
  Mvd = 0x2,
  And = 0x3,
  Or = 0x4,
  Xor = 0x5,
  Add = 0x6,
  Sub = 0x7,
  Shr = 0x8,
  Shl = 0x9,
  Sha = 0xA,
  Jl = 0xB,
  Jg = 0xC,
  Jb = 0xD,
  Ja = 0xE,
  Jq = 0xF,
}

#[derive(Copy, Clone)]
pub struct Opcode {
  base: BaseOp,
  imm: bool,
  bits16: bool,
}

impl Opcode {
  pub fn from_str(s: &str) -> Option<Opcode> {
    Some(match s {
      "mvi" => Opcode { base: BaseOp::Mvi, bits16: true, imm: false },
      "mvib" => Opcode { base: BaseOp::Mvi, bits16: false, imm: false },
      "ldi" => Opcode { base: BaseOp::Mvi, bits16: true, imm: true },
      "ldbi" => Opcode { base: BaseOp::Mvi, bits16: false, imm: true },

      "mv" => Opcode { base: BaseOp::Mv, bits16: true, imm: false },
      "mvb" => Opcode { base: BaseOp::Mv, bits16: false, imm: false },
      "ld" => Opcode { base: BaseOp::Mv, bits16: true, imm: true },
      "ldb" => Opcode { base: BaseOp::Mv, bits16: false, imm: true },

      "mvd" => Opcode { base: BaseOp::Mvd, bits16: true, imm: false },
      "mvdb" => Opcode { base: BaseOp::Mvd, bits16: false, imm: false },
      "ldd" => Opcode { base: BaseOp::Mvd, bits16: true, imm: true },
      "ldbd" => Opcode { base: BaseOp::Mvd, bits16: false, imm: true },

      "nd" => Opcode { base: BaseOp::And, bits16: true, imm: false },
      "ndb" => Opcode { base: BaseOp::And, bits16: false, imm: false },
      "ndi" => Opcode { base: BaseOp::And, bits16: true, imm: true },
      "ndbi" => Opcode { base: BaseOp::And, bits16: false, imm: true },

      "or" => Opcode { base: BaseOp::Or, bits16: true, imm: false },
      "orb" => Opcode { base: BaseOp::Or, bits16: false, imm: false },
      "ori" => Opcode { base: BaseOp::Or, bits16: true, imm: true },
      "orbi" => Opcode { base: BaseOp::Or, bits16: false, imm: true },

      "xr" => Opcode { base: BaseOp::Xor, bits16: true, imm: false },
      "xrb" => Opcode { base: BaseOp::Xor, bits16: false, imm: false },
      "xri" => Opcode { base: BaseOp::Xor, bits16: true, imm: true },
      "xrbi" => Opcode { base: BaseOp::Xor, bits16: false, imm: true },

      "ad" => Opcode { base: BaseOp::Add, bits16: true, imm: false },
      "adb" => Opcode { base: BaseOp::Add, bits16: false, imm: false },
      "adi" => Opcode { base: BaseOp::Add, bits16: true, imm: true },
      "adbi" => Opcode { base: BaseOp::Add, bits16: false, imm: true },

      "sb" => Opcode { base: BaseOp::Sub, bits16: true, imm: false },
      "sbb" => Opcode { base: BaseOp::Sub, bits16: false, imm: false },
      "sbi" => Opcode { base: BaseOp::Sub, bits16: true, imm: true },
      "sbbi" => Opcode { base: BaseOp::Sub, bits16: false, imm: true },

      "sr" => Opcode { base: BaseOp::Shr, bits16: true, imm: false },
      "srb" => Opcode { base: BaseOp::Shr, bits16: false, imm: false },
      "sri" => Opcode { base: BaseOp::Shr, bits16: true, imm: true },
      "srbi" => Opcode { base: BaseOp::Shr, bits16: false, imm: true },

      "sl" => Opcode { base: BaseOp::Shl, bits16: true, imm: false },
      "slb" => Opcode { base: BaseOp::Shl, bits16: false, imm: false },
      "sli" => Opcode { base: BaseOp::Shl, bits16: true, imm: true },
      "slbi" => Opcode { base: BaseOp::Shl, bits16: false, imm: true },

      "sa" => Opcode { base: BaseOp::Sha, bits16: true, imm: false },
      "sab" => Opcode { base: BaseOp::Sha, bits16: false, imm: false },
      "sai" => Opcode { base: BaseOp::Sha, bits16: true, imm: true },
      "sabi" => Opcode { base: BaseOp::Sha, bits16: false, imm: true },

      "jl" => Opcode { base: BaseOp::Jl, bits16: true, imm: false },
      "jle" => Opcode { base: BaseOp::Jl, bits16: false, imm: false },
      "jli" => Opcode { base: BaseOp::Jl, bits16: true, imm: true },
      "jlei" => Opcode { base: BaseOp::Jl, bits16: false, imm: true },

      "jg" => Opcode { base: BaseOp::Jg, bits16: true, imm: false },
      "jge" => Opcode { base: BaseOp::Jg, bits16: false, imm: false },
      "jgi" => Opcode { base: BaseOp::Jg, bits16: true, imm: true },
      "jgei" => Opcode { base: BaseOp::Jg, bits16: false, imm: true },

      "jb" => Opcode { base: BaseOp::Jb, bits16: true, imm: false },
      "jbe" => Opcode { base: BaseOp::Jb, bits16: false, imm: false },
      "jbi" => Opcode { base: BaseOp::Jb, bits16: true, imm: true },
      "jbei" => Opcode { base: BaseOp::Jb, bits16: false, imm: true },

      "ja" => Opcode { base: BaseOp::Ja, bits16: true, imm: false },
      "jae" => Opcode { base: BaseOp::Ja, bits16: false, imm: false },
      "jai" => Opcode { base: BaseOp::Ja, bits16: true, imm: true },
      "jaei" => Opcode { base: BaseOp::Ja, bits16: false, imm: true },

      "jq" => Opcode { base: BaseOp::Jq, bits16: true, imm: false },
      "jnq" => Opcode { base: BaseOp::Jq, bits16: false, imm: false },
      "jqi" => Opcode { base: BaseOp::Jq, bits16: true, imm: true },
      "jnqi" => Opcode { base: BaseOp::Jq, bits16: false, imm: true },

      _ => return None,
    })
  }

  pub fn to_u16(self) -> u16 {
    self.base as u16
      | (self.bits16 as u16) << 4
      | (self.imm as u16) << 5
  }

  pub fn size(&self) -> u16 {
    if self.is_arith() {
      2
    } else {
      3
    }
  }

  pub fn is_arith(&self) -> bool {
    (self.base as u16) < (BaseOp::Jl as u16)
  }

  fn args(&self) -> &'static [OpArg] {
    static ARGS: &'static [OpArg] = &[
      OpArg {
        var: OpArgVar::MacroArg(0),
        pos: lexer.compiler_defined_pos(),
      },
      OpArg {
        var: OpArgVar::MacroArg(1),
        pos: lexer.compiler_defined_pos(),
      },
      OpArg {
        var: OpArgVar::MacroArg(2),
        pos: lexer.compiler_defined_pos(),
      },
    ];
    if self.is_arith() {
    }
  }
}

pub struct Parser {
  // TODO(ubsan): rename to inst_buffer
  inst_buffer: Vec<Instruction>,
  inst_buffer_idx: usize,
  inst_offset: u16,
  directives: Vec<Directive>,
  labels: HashMap<String, u16>,
  macros: HashMap<String, (u16, Vec<(Opcode, Vec<OpArg>)>)>,
  idx: usize,
}

impl Parser {
  pub fn new(filename: &str) -> Self {
    // compiler_defined_pos
    macro_rules! macro_op_arg {
      ($lexer:expr, $var:ident) => (
        OpArg {
          var: OpArgVar::$var,
          pos: $lexer.compiler_defined_pos(),
        }
      );
      ($lexer:expr, $var:ident ($($arg:tt)*)) => (
        OpArg {
          var: OpArgVar::$var($($arg)*),
          pos: $lexer.compiler_defined_pos(),
        }
      );
    }
    let path: PathBuf = match Path::new(filename).canonicalize() {
      Ok(c) => c,
      Err(_) => error_np!("Unable to open file: {}", filename),
    };
    let lexer = Lexer::new(&path);
    let mut this = Parser {
      inst_buffer: Vec::new(),
      inst_buffer_idx: 0,
      inst_offset: CODE_START,
      directives: Vec::new(),
      labels: hashmap! {
        "ip".to_owned() => REG_IP,
        "sp".to_owned() => REG_SP,
        "bp".to_owned() => REG_BP,
        "sc0".to_owned() => REG_SC0,
        "sc1".to_owned() => REG_SC1,
        "sc2".to_owned() => REG_SC2,
        "sc3".to_owned() => REG_SC3,
      },
      macros: hashmap! {},
      idx: 0,
    };

    this.get_directives(&mut vec![path], lexer);

    // normal labels
    let mut inst_offset = CODE_START;
    for dir in &this.directives {
      match dir.var {
        DirectiveVar::Label(ref s, ref _public) => {
          // NOTE(ubsan): can optimize this to mem::replace(String::new())
          match this.labels.insert(s.clone(), inst_offset) {
            Some(_) => {
              error!(dir.pos, "Attempted to redefine label: {}", s);
            }
            None => {}
          }
        }
        DirectiveVar::Op(ref op, _) =>
          inst_offset += this.size_of_op_str(&dir.pos, op),
        DirectiveVar::Const(..) => {}
        DirectiveVar::Data(ref data) => inst_offset += (data.len() * 2) as u16,
        DirectiveVar::ByteData(ref data) => inst_offset += data.len() as u16,
        DirectiveVar::Public(_) => {
          // TODO(ubsan): silently ignored for now
        },
        DirectiveVar::Import(_, _) => {},
        DirectiveVar::Macro{..} => unimplemented!(),
      }
    }

    // equ constants
    let mut inst_offset = CODE_START;
    for dir in &this.directives {
      match dir.var {
        DirectiveVar::Label(..) => {}
        DirectiveVar::Op(ref op, _) =>
          inst_offset += this.size_of_op_str(&dir.pos, op),
        DirectiveVar::Const(ref s, ref arg, ref _public) => {
          let n = arg.evaluate(&this.labels, &[], inst_offset);
          match this.labels.insert(s.clone(), n) {
            Some(s) => error!(
              arg.pos, "Attempted to redefine label: {}", s
            ),
            None => {},
          }
        }
        DirectiveVar::Data(ref data) => inst_offset += (data.len() * 2) as u16,
        DirectiveVar::ByteData(ref data) => inst_offset += data.len() as u16,
        DirectiveVar::Public(_) => {
          // TODO(ubsan): silently ignored for now
        },
        DirectiveVar::Import(_, _) => {},
        DirectiveVar::Macro{..} => unimplemented!(),
      }
    }

    this
  }

  fn get_directives(&mut self, imports: &mut Vec<PathBuf>, mut lexer: Lexer) {
    use std::path::{Path, PathBuf};
    fn make_path(cur_path: &Path, pos: &Position, vec: Vec<String>) -> PathBuf {
      assert!(!vec.is_empty(), "ICE: DirectiveVar::Import had an empty Vec");
      let mut filename = PathBuf::new();
      for dir in &vec {
        filename.push(dir);
      }
      filename.set_extension("asm");

      let mut ret = PathBuf::from(cur_path);
      ret.set_file_name(filename);
      match ret.canonicalize() {
        Ok(c) => c,
        Err(_) => error!(pos, "failure to open import: {}", {
          let mut tmp = vec.iter().fold(String::new(), |mut s, el| {
            s.push_str(&el); s.push('.'); s
          });
          tmp.pop();
          tmp
        })
      }
    }
    fn push_unique(vec: &mut Vec<PathBuf>, to_push: PathBuf) -> bool {
      if vec.contains(&to_push) {
        false
      } else {
        vec.push(to_push);
        true
      }
    }
    let index = imports.len() - 1;
    while let Some(dir) = lexer.next_directive() {
      if let DirectiveVar::Import(path, _) = dir.var {
        let path = make_path(&imports[index], &dir.pos, path);
        if push_unique(imports, path) {
          let new_lexer = lexer.new_file_lexer(&imports[imports.len() - 1]);
          self.get_directives(imports, new_lexer);
        }
      } else {
        self.directives.push(dir);
      }
    }
  }

  fn size_of_op_str(&self, pos: &Position, op: &str) -> u16 {
    match Opcode::from_str(op) {
      Some(op) => op.size(),
      None => match self.macros.get(op) {
        Some(&(_, ref ops)) => {
          let mut acc = 0;
          for &(ref op, ref _args) in ops {
            acc += op.size();
          }
          acc
        }
        None => error!(pos, "Unknown opcode: {}", op),
      }
    }
  }

  fn next_directive(&mut self) -> Option<Directive> {
    if self.idx < self.directives.len() {
      let ret = ::std::mem::replace(&mut self.directives[self.idx], Directive {
        var: DirectiveVar::Label(String::new(), Public::Private),
        pos: Position::empty(),
      });
      self.idx += 1;
      Some(ret)
    } else {
      None
    }
  }
}

impl Iterator for Parser {
  type Item = Instruction;

  fn next(&mut self) -> Option<Instruction> {
    fn arith(
      this: &Parser, op: Opcode, args: &[OpArg], mac_args: &[OpArg],
    ) -> (Instruction, u16) {
      let reg = args[0].evaluate(&this.labels, mac_args, this.inst_offset);
      if reg >= CODE_START {
        error!(
          mac_args[0].pos, "Register memory is out of range: {}", reg,
        );
      }
      let imm = args[1].evaluate(&this.labels, mac_args, this.inst_offset);
      (Instruction::Arith {
        op: op,
        reg: reg,
        imm: imm,
      }, 4)
    }
    fn jump(
      this: &Parser, op: Opcode, args: &[OpArg], mac_args: &[OpArg],
    ) -> (Instruction, u16) {
      let reg = args[0].evaluate(&this.labels, mac_args, this.inst_offset);
      if reg >= CODE_START {
        error!(
          mac_args[0].pos, "Register memory is out of range: {}", reg,
        );
      }
      let imm = args[1].evaluate(&this.labels, mac_args, this.inst_offset);
      let label = args[2].evaluate(&this.labels, mac_args, this.inst_offset);
      (Instruction::Jump {
        op: op,
        reg: reg,
        imm: imm,
        label: label,
      }, 6)
    }
    fn data(this: &Parser, data: Vec<OpArg>) -> (Instruction, u16) {
      let mut data_num = Vec::new();
      // heh. datum.
      for datum in data {
        data_num.push(datum.evaluate(&this.labels, &[], this.inst_offset))
      }
      let offset = (data_num.len() * 2) as u16;
      (Instruction::Data(data_num), offset)
    }
    fn byte_data(this: &Parser, data: Vec<OpArg>) -> (Instruction, u16) {
      let mut data_num = Vec::new();
      for datum in data {
        data_num.push(datum.evaluate_u8(&this.labels, &[], this.inst_offset))
      }
      if data_num.len() % 2 != 0 {
        data_num.push(0);
      }
      let offset = data_num.len() as u16;
      (Instruction::ByteData(data_num), offset)
    }
    // the u16 is the inst_offset to add
    fn opcode(
      this: &Parser, op: Opcode, args: &[OpArg], mac_args: &[OpArg],
    ) -> (Instruction, u16) {
      if op.is_arith() {
        arith(this, op, args, &mac_args)
      } else {
        jump(this, op, args, &mac_args)
      }
    }

    let op = match self.inst_buffer.get_mut(self.inst_buffer_idx) {
      Some(inst) => Some(
        ::std::mem::replace(inst, Instruction::Data(Vec::new()))
      ),
      None => None,
    };
    if let Some(op) = op {
      self.inst_buffer_idx += 1;
      return Some(op);
    } else {
      self.inst_buffer_idx = 0;
      self.inst_buffer.clear();
    }
    if let Some(dir) = self.next_directive() {
      match dir.var {
        DirectiveVar::Op(op, mac_args) => {
          if let Some(op) = Opcode::from_str(&op) {
            let (op, offset) = opcode(self, op, , &mac_args);
            self.inst_offset += offset;
            self.inst_buffer.push(op);
          } else {
            match self.macros.get(&op) {
              Some(&(ref size, ref ops)) => {
                if (mac_args.len() as u16) != *size {
                  error!(
                    dir.pos,
                    "Invalid number of args to {}; expected {}, found {}",
                    op,
                    size,
                    mac_args.len(),
                  )
                }
                for &(ref op, ref args) in ops {
                  let (op, offset) = opcode(self, *op, args, &mac_args);
                  self.inst_offset += offset;
                  self.inst_buffer.push(op);
                }
              },
              None => error!(dir.pos, "Unknown opcode"),
            }
          }
          if let Some(inst) = self.inst_buffer.get_mut(0) {
            self.inst_buffer_idx = 1;
            return Some(
              ::std::mem::replace(inst, Instruction::Data(Vec::new()))
            );
          }
          self.next()
        },
        DirectiveVar::Data(nums) => {
          let (data, offset) = data(self, nums);
          self.inst_offset += offset;
          Some(data)
        },
        DirectiveVar::ByteData(nums) => {
          let (data, offset) = byte_data(self, nums);
          self.inst_offset += offset;
          Some(data)
        },
        DirectiveVar::Label(..) | DirectiveVar::Const(..) => {
          while let Some(dir) = self.directives.get(self.idx) {
            match dir.var {
              DirectiveVar::Label(..) | DirectiveVar::Const(..) =>
                self.idx += 1,
              _ => break,
            }
          }
          self.next()
        },
        DirectiveVar::Public(_) => {
          // TODO(ubsan): silently ignored for now
          // there is no public|private distinction
          self.next()
        }
        DirectiveVar::Import(_, _) => {
          // imports aren't dealt with here
          self.next()
        },
        DirectiveVar::Macro{..} => unimplemented!(),
      }
    } else {
      None
    }
  }
}
