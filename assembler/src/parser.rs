use std::collections::HashMap;
use {Opcode, OpcodeVariant};

use lexer::{self, Directive, DirectiveVar, Lexer, OpArg, OpArgVar};

const INST_OFFSET_BASE: u16 = 0x1000;

const REG_IP: u16 = 0x0;
const REG_SP: u16 = 0x1;
const REG_BP: u16 = 0x2;
const REG_SC: u16 = 0x3;
const REG_SC2: u16 = 0x4;

#[derive(Copy, Clone)]
enum BaseOp {
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
  JumpGreater,
  JumpLesser,
  JumpEqual,
}

pub struct Parser {
  op_buffer: Vec<Opcode>,
  op_buffer_idx: usize,
  inst_offset: u16,
  directives: Vec<Directive>,
  labels: HashMap<String, u16>,
  macros: HashMap<String, (u16, Vec<(BaseOp, Vec<OpArg>)>)>,
  idx: usize,
}

impl Parser {
  pub fn new(input: Vec<u8>) -> Self {
    macro_rules! macro_op_arg {
      ($var:ident) => (
        OpArg {
          variant: OpArgVar::$var,
          line: 0,
          offset: 0,
        }
      );
      ($var:ident ($($arg:tt)*)) => (
        OpArg {
          variant: OpArgVar::$var($($arg)*),
          line: 0,
          offset: 0,
        }
      );
    }
    let mut lexer = Lexer::new(input);
    let mut this = Parser {
      op_buffer: Vec::new(),
      op_buffer_idx: 0,
      inst_offset: INST_OFFSET_BASE,
      directives: Vec::new(),
      labels: hashmap! {
        "IP".to_owned() => REG_IP,
        "SP".to_owned() => REG_SP,
        "BP".to_owned() => REG_BP,
        "SC".to_owned() => REG_SC,
      },
      macros: hashmap! {
        "MI".to_owned() => (2, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "MV".to_owned() => (2, vec![
          (BaseOp::Move, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1))
          ])
        ]),
        "MD".to_owned() => (2, vec![
          (BaseOp::MoveDeref, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "LD".to_owned() => (2, vec![
          (BaseOp::Load, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1))
          ])
        ]),
        "ST".to_owned() => (2, vec![
          (BaseOp::Store, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "AD".to_owned() => (2, vec![
          (BaseOp::Add, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "SB".to_owned() => (2, vec![
          (BaseOp::Sub, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "ND".to_owned() => (2, vec![
          (BaseOp::And, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "OR".to_owned() => (2, vec![
          (BaseOp::Or, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "XR".to_owned() => (2, vec![
          (BaseOp::Xor, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "SR".to_owned() => (2, vec![
          (BaseOp::ShiftRight, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "SL".to_owned() => (2, vec![
          (BaseOp::ShiftLeft, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "SA".to_owned() => (2, vec![
          (BaseOp::ShiftArithmetic, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(1)),
          ])
        ]),
        "JG".to_owned() => (3, vec![
          (BaseOp::JumpGreater, vec![
            macro_op_arg!(MacroArg(0)),
            macro_op_arg!(MacroArg(1)),
            macro_op_arg!(MacroArg(2)),
          ])
        ]),
        "JL".to_owned() => (3, vec![
          (BaseOp::JumpLesser, vec![
            macro_op_arg!(MacroArg(0)),
            macro_op_arg!(MacroArg(1)),
            macro_op_arg!(MacroArg(2)),
          ])
        ]),
        "JQ".to_owned() => (3, vec![
          (BaseOp::JumpEqual, vec![
            macro_op_arg!(MacroArg(0)),
            macro_op_arg!(MacroArg(1)),
            macro_op_arg!(MacroArg(2)),
          ])
        ]),
        "HF".to_owned() => (0, vec![
          (BaseOp::JumpEqual, vec![
            macro_op_arg!(Number(REG_IP)),
            macro_op_arg!(Number(REG_IP)),
            macro_op_arg!(Here),
          ]),
        ]),
        "JM".to_owned() => (1, vec![
          (BaseOp::Move, vec![
            macro_op_arg!(Number(REG_IP)), macro_op_arg!(MacroArg(0))]),
        ]),
        "JI".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_IP)), macro_op_arg!(MacroArg(0))]),
        ]),
        "INC".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(Number(1))]),
          (BaseOp::Add, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(Number(REG_SC))]),
        ]),
        "DEC".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(Number(1))]),
          (BaseOp::Sub, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(Number(REG_SC))]),
        ]),
        "NEG".to_owned() => (1, vec![
          (BaseOp::Move, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(MacroArg(0))]),
          (BaseOp::Xor, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(MacroArg(0))]),
          (BaseOp::Move, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(Number(REG_SC))]),
        ]),
        "ADI".to_owned() => (2, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(MacroArg(1))]),
          (BaseOp::Add, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(Number(REG_SC))]),
        ]),
        "SBI".to_owned() => (2, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(MacroArg(1))]),
          (BaseOp::Sub, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(Number(REG_SC))]),
        ]),
        "PUSH".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(Number(1))
          ]),
          (BaseOp::Add, vec![
            macro_op_arg!(Number(REG_SP)), macro_op_arg!(Number(REG_SC))
          ]),
          (BaseOp::Load, vec![
            macro_op_arg!(Number(REG_SP)), macro_op_arg!(MacroArg(0))
          ]),
        ]),
        "POP".to_owned() => (1, vec![
          (BaseOp::MoveDeref, vec![
            macro_op_arg!(MacroArg(0)), macro_op_arg!(Number(REG_SP))
          ]),
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(Number(1))
          ]),
          (BaseOp::Sub, vec![
            macro_op_arg!(Number(REG_SP)), macro_op_arg!(Number(REG_SC))
          ]),
        ]),
        "CALL".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(Number(1))
          ]),
          (BaseOp::Add, vec![
            macro_op_arg!(Number(REG_SP)), macro_op_arg!(Number(REG_SC))
          ]),
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)),
            macro_op_arg!(ArithOp(
              lexer::ArithOp::Add,
              Box::new(macro_op_arg!(Here)),
              Box::new(macro_op_arg!(Number(6))),
            ))
          ]),
          (BaseOp::Load, vec![
            macro_op_arg!(Number(REG_SP)), macro_op_arg!(Number(REG_SC))
          ]),
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_IP)), macro_op_arg!(MacroArg(0))
          ])
        ]),
        "RET".to_owned() => (0, vec![
          (BaseOp::MoveDeref, vec![
            macro_op_arg!(Number(REG_SC2)), macro_op_arg!(Number(REG_SP))
          ]),
          (BaseOp::MoveImmediate, vec![
            macro_op_arg!(Number(REG_SC)), macro_op_arg!(Number(1))
          ]),
          (BaseOp::Sub, vec![
            macro_op_arg!(Number(REG_SP)), macro_op_arg!(Number(REG_SC))
          ]),
          (BaseOp::Move, vec![
            macro_op_arg!(Number(REG_IP)), macro_op_arg!(Number(REG_SC2))
          ])
        ]),
      },
      idx: 0,
    };

    while let Some(dir) = lexer.next_directive() {
      this.directives.push(dir);
    }

    // normal labels
    let mut inst_offset = INST_OFFSET_BASE;
    for dir in &this.directives {
      match dir.variant {
        DirectiveVar::Label(ref s) => {
          // NOTE(ubsan): can optimize this to mem::replace(String::new())
          match this.labels.insert(s.clone(), inst_offset) {
            Some(s) => {
              error!(
                dir.line, dir.offset, "Attempted to redefine label: {}", s
              );
            }
            None => {}
          }
        }
        DirectiveVar::Op(ref op, _) =>
          inst_offset += this.size_of_op_str(dir.line, dir.offset, op),
        DirectiveVar::Const(..) => {}
        DirectiveVar::Data(ref data) => inst_offset += data.len() as u16,
        DirectiveVar::Macro{..} => unimplemented!(),
      }
    }

    // equ constants
    let mut inst_offset = INST_OFFSET_BASE;
    for dir in &this.directives {
      match dir.variant {
        DirectiveVar::Label(..) => {}
        DirectiveVar::Op(ref op, _) =>
          inst_offset += this.size_of_op_str(dir.line, dir.offset, op),
        DirectiveVar::Const(ref s, ref arg) => {
          let n = arg.evaluate(&this.labels, &[], inst_offset);
          match this.labels.insert(s.clone(), n) {
            Some(s) => error!(
              arg.line, arg.offset, "Attempted to redefine label: {}", s
            ),
            None => {},
          }
        }
        DirectiveVar::Data(ref data) => inst_offset += data.len() as u16,
        DirectiveVar::Macro{..} => unimplemented!(),
      }
    }

    this
  }

  fn size_of_op_str(&self, line: usize, offset: usize, op: &str) -> u16 {
    match self.macros.get(op) {
      Some(&(_, ref ops)) => {
        let mut acc = 0;
        for &(ref op, ref _args) in ops {
          acc += self.size_of_op(*op);
        }
        acc
      }
      None => error!(line, offset, "Unknown opcode: {}", op),
    }
  }

  fn size_of_op(&self, op: BaseOp) -> u16 {
    match op {
      BaseOp::MoveImmediate | BaseOp::Move | BaseOp::MoveDeref | BaseOp::Load
      | BaseOp::Store | BaseOp::Add | BaseOp::Sub | BaseOp::And | BaseOp::Or
      | BaseOp::Xor | BaseOp::ShiftRight | BaseOp::ShiftLeft
      | BaseOp::ShiftArithmetic => 2,
      BaseOp::JumpGreater | BaseOp::JumpLesser | BaseOp::JumpEqual => 3,
    }
  }

  fn next_directive(&mut self) -> Option<Directive> {
    if self.idx < self.directives.len() {
      let ret = ::std::mem::replace(&mut self.directives[self.idx], Directive {
        variant: DirectiveVar::Label(String::new()),
        line: 0,
        offset: 0,
      });
      self.idx += 1;
      Some(ret)
    } else {
      None
    }
  }
}

impl Iterator for Parser {
  type Item = Opcode;

  fn next(&mut self) -> Option<Opcode> {
    fn arith(
      this: &Parser,
      op: OpcodeVariant,
      args: &[OpArg],
      mac_args: &[OpArg],
    ) -> (Opcode, u16) {
      let reg = args[0].evaluate(&this.labels, mac_args, this.inst_offset);
      if reg >= 0x1000 {
        error!(
          mac_args[0].line,
          mac_args[0].offset,
          "Register memory is out of range: {}",
          reg,
        );
      }
      let num = args[1].evaluate(&this.labels, mac_args, this.inst_offset);
      (Opcode {
        variant: op,
        reg: reg,
        num: num,
      }, 2)
    }
    fn jump(
      this: &Parser,
      op: fn(u16) -> OpcodeVariant,
      args: &[OpArg],
      mac_args: &[OpArg],
    ) -> (Opcode, u16) {
      let reg = args[0].evaluate(&this.labels, mac_args, this.inst_offset);
      if reg >= 0x1000 {
        error!(
          mac_args[0].line,
          mac_args[0].offset,
          "Register memory is out of range: {}",
          reg,
        );
      }
      let num = args[1].evaluate(&this.labels, mac_args, this.inst_offset);
      let label = args[2].evaluate(&this.labels, mac_args, this.inst_offset);
      (Opcode {
        variant: op(label),
        reg: reg,
        num: num,
      }, 3)
    }
    fn data(this: &Parser, data: Vec<OpArg>) -> (Opcode, u16) {
      let mut data_num = Vec::new();
      // heh. datum.
      for datum in data {
        data_num.push(datum.evaluate(&this.labels, &[], this.inst_offset))
      }
      let offset = data_num.len() as u16;
      (Opcode {
        variant: OpcodeVariant::Data(data_num),
        reg: 0,
        num: 0,
      }, offset)
    }
    // the u16 is the inst_offset to add
    fn opcode(
      this: &Parser, op: &BaseOp, args: &[OpArg], mac_args: &[OpArg],
    ) -> (Opcode, u16) {
      match *op {
        BaseOp::MoveImmediate =>
          arith(this, OpcodeVariant::MoveImmediate, args, &mac_args),
        BaseOp::Move => arith(this, OpcodeVariant::Move, args, &mac_args),
        BaseOp::MoveDeref =>
          arith(this, OpcodeVariant::MoveDeref, args, &mac_args),
        BaseOp::Load => arith(this, OpcodeVariant::Load, args, &mac_args),
        BaseOp::Store => arith(this, OpcodeVariant::Store, args, &mac_args),
        BaseOp::Add => arith(this, OpcodeVariant::Add, args, &mac_args),
        BaseOp::Sub => arith(this, OpcodeVariant::Sub, args, &mac_args),
        BaseOp::And => arith(this, OpcodeVariant::And, args, &mac_args),
        BaseOp::Or => arith(this, OpcodeVariant::Or, args, &mac_args),
        BaseOp::Xor => arith(this, OpcodeVariant::Xor, args, &mac_args),
        BaseOp::ShiftRight =>
          arith(this, OpcodeVariant::ShiftRight, args, &mac_args),
        BaseOp::ShiftLeft =>
          arith(this, OpcodeVariant::ShiftLeft, args, &mac_args),
        BaseOp::ShiftArithmetic =>
          arith(this, OpcodeVariant::ShiftArithmetic, args, &mac_args),
        BaseOp::JumpGreater =>
          jump(this, OpcodeVariant::JumpGreater, args, &mac_args),
        BaseOp::JumpLesser =>
          jump(this, OpcodeVariant::JumpLesser, args, &mac_args),
        BaseOp::JumpEqual =>
          jump(this, OpcodeVariant::JumpEqual, args, &mac_args),
      }
    }

    let op = match self.op_buffer.get_mut(self.op_buffer_idx) {
      Some(op) => Some(
        ::std::mem::replace(op, Opcode {
          variant: OpcodeVariant::MoveImmediate,
          reg: 0,
          num: 0,
        })
      ),
      None => None,
    };
    if let Some(op) = op {
      self.op_buffer_idx += 1;
      return Some(op);
    } else {
      self.op_buffer_idx = 0;
      self.op_buffer.clear();
    }
    if let Some(dir) = self.next_directive() {
      match dir.variant {
        DirectiveVar::Op(op, mac_args) => {
          match self.macros.get(&op) {
            Some(&(ref size, ref ops)) => {
              if (mac_args.len() as u16) != *size {
                error!(
                  dir.line,
                  dir.offset,
                  "Invalid number of args to {}; expected {}, found {}",
                  op,
                  size,
                  mac_args.len(),
                )
              }
              for &(ref op, ref args) in ops {
                let (op, offset) = opcode(self, op, args, &mac_args);
                self.inst_offset += offset;
                self.op_buffer.push(op);
              }
            },
            None => error!(dir.line, dir.offset, "Unknown opcode"),
          }
          if let Some(op) = self.op_buffer.get_mut(0) {
            self.op_buffer_idx = 1;
            return Some(::std::mem::replace(op, Opcode {
              variant: OpcodeVariant::MoveImmediate,
              reg: 0,
              num: 0,
            }));
          }
          self.next()
        },
        DirectiveVar::Data(nums) => {
          let (data, offset) = data(self, nums);
          self.inst_offset += offset;
          Some(data)
        }
        DirectiveVar::Label(..) | DirectiveVar::Const(..) => {
          while let Some(dir) = self.directives.get(self.idx) {
            match dir.variant {
              DirectiveVar::Label(..) | DirectiveVar::Const(..) =>
                self.idx += 1,
              _ => break,
            }
          }
          self.next()
        },
        DirectiveVar::Macro{..} => unimplemented!(),
      }
    } else {
      None
    }
  }
}
