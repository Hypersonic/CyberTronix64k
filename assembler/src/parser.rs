use std::collections::HashMap;
use {Opcode, OpcodeVariant};

use lexer::{Directive, Lexer, OpArg};

const INST_OFFSET_BASE: u16 = 0x1000;

const REG_IP: u16 = 0x0;
const REG_SP: u16 = 0x1;
const REG_BP: u16 = 0x2;
const REG_SC: u16 = 0x3;

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
          (BaseOp::MoveImmediate, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "MV".to_owned() => (2, vec![
          (BaseOp::Move, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "MD".to_owned() => (2, vec![
          (BaseOp::MoveDeref, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "LD".to_owned() => (2, vec![
          (BaseOp::Load, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "ST".to_owned() => (2, vec![
          (BaseOp::Store, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "AD".to_owned() => (2, vec![
          (BaseOp::Add, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "SB".to_owned() => (2, vec![
          (BaseOp::Sub, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "ND".to_owned() => (2, vec![
          (BaseOp::And, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "OR".to_owned() => (2, vec![
          (BaseOp::Or, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "XR".to_owned() => (2, vec![
          (BaseOp::Xor, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "SR".to_owned() => (2, vec![
          (BaseOp::ShiftRight, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "SL".to_owned() => (2, vec![
          (BaseOp::ShiftLeft, vec![OpArg::MacroArg(0), OpArg::MacroArg(1)])
        ]),
        "SA".to_owned() => (2, vec![
          (BaseOp::ShiftArithmetic, vec![
            OpArg::MacroArg(0), OpArg::MacroArg(1),
          ])
        ]),
        "JG".to_owned() => (3, vec![
          (BaseOp::JumpGreater, vec![
            OpArg::MacroArg(0), OpArg::MacroArg(1), OpArg::MacroArg(2),
          ])
        ]),
        "JL".to_owned() => (3, vec![
          (BaseOp::JumpLesser, vec![
            OpArg::MacroArg(0), OpArg::MacroArg(1), OpArg::MacroArg(2),
          ])
        ]),
        "JQ".to_owned() => (3, vec![
          (BaseOp::JumpEqual, vec![
            OpArg::MacroArg(0), OpArg::MacroArg(1), OpArg::MacroArg(2),
          ])
        ]),
        "HF".to_owned() => (0, vec![
          (BaseOp::JumpEqual, vec![
            OpArg::Number(REG_IP), OpArg::Number(REG_IP), OpArg::Here]),
        ]),
        "JM".to_owned() => (1, vec![
          (BaseOp::Move, vec![
            OpArg::Number(REG_IP), OpArg::MacroArg(0)]),
        ]),
        "JI".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            OpArg::Number(REG_IP), OpArg::MacroArg(0)]),
        ]),
        "INC".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            OpArg::Number(REG_SC), OpArg::Number(1)]),
          (BaseOp::Add, vec![
            OpArg::MacroArg(0), OpArg::Number(REG_SC)]),
        ]),
        "DEC".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            OpArg::Number(REG_SC), OpArg::Number(1)]),
          (BaseOp::Sub, vec![
            OpArg::MacroArg(0), OpArg::Number(REG_SC)]),
        ]),
        "NEG".to_owned() => (1, vec![
          (BaseOp::Move, vec![
            OpArg::Number(REG_SC), OpArg::MacroArg(0)]),
          (BaseOp::Xor, vec![
            OpArg::MacroArg(0), OpArg::MacroArg(0)]),
          (BaseOp::Move, vec![
            OpArg::MacroArg(0), OpArg::Number(REG_SC)]),
        ]),
        "ADI".to_owned() => (2, vec![
          (BaseOp::MoveImmediate, vec![
            OpArg::Number(REG_SC), OpArg::MacroArg(1)]),
          (BaseOp::Add, vec![
            OpArg::MacroArg(0), OpArg::Number(REG_SC)]),
        ]),
        "SBI".to_owned() => (2, vec![
          (BaseOp::MoveImmediate, vec![
            OpArg::Number(REG_SC), OpArg::MacroArg(1)]),
          (BaseOp::Sub, vec![
            OpArg::MacroArg(0), OpArg::Number(REG_SC)]),
        ]),
        "PUSH".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            OpArg::Number(REG_SC), OpArg::Number(1)]),
          (BaseOp::Add, vec![
            OpArg::Number(REG_SP), OpArg::Number(REG_SC)]),
          (BaseOp::Load, vec![
            OpArg::Number(REG_SP), OpArg::MacroArg(0)]),
        ]),
        "POP".to_owned() => (1, vec![
          (BaseOp::MoveDeref, vec![
            OpArg::MacroArg(0), OpArg::Number(REG_SP)]),
          (BaseOp::MoveImmediate, vec![
            OpArg::Number(REG_SC), OpArg::Number(1)]),
          (BaseOp::Sub, vec![
            OpArg::Number(REG_SP), OpArg::Number(REG_SC)]),
        ]),
      },
      idx: 0,
    };

    while let Some(dir) = lexer.next_directive() {
      this.directives.push(dir);
    }

    // normal labels
    let mut inst_offset = INST_OFFSET_BASE;
    for directive in &this.directives {
      match *directive {
        Directive::Label(ref s) => {
          // NOTE(ubsan): can optimize this to mem::replace(String::new())
          match this.labels.insert(s.clone(), inst_offset) {
            Some(s) => {
              abort!("Attempted to redefine label: {}", s);
            }
            None => {}
          }
        }
        Directive::Op(ref op, _) => inst_offset += this.size_of_op_str(op),
        Directive::Const(..) => {}
        Directive::Data(ref data) => inst_offset += data.len() as u16,
        Directive::Macro{..} => unimplemented!(),
      }
    }

    // equ constants
    let mut inst_offset = INST_OFFSET_BASE;
    for directive in &this.directives {
      match *directive {
        Directive::Label(..) => {}
        Directive::Op(ref op, _) => inst_offset += this.size_of_op_str(op),
        Directive::Const(ref s, ref arg) => {
          let n = match *arg {
            OpArg::Number(n) => n,
            OpArg::Label(ref label) => match this.labels.get(label) {
              Some(&n) => n,
              None => abort!("Unknown label: {}", label),
            },
            OpArg::Here => inst_offset,
            OpArg::MacroArg(_) => unreachable!(),
          };
          match this.labels.insert(s.clone(), n) {
            Some(s) => abort!("Attempted to redefine label: {}", s),
            None => {},
          }
        }
        Directive::Data(ref data) => inst_offset += data.len() as u16,
        Directive::Macro{..} => unimplemented!(),
      }
    }

    this
  }

  fn size_of_op_str(&self, op: &str) -> u16 {
    match self.macros.get(op) {
      Some(&(_, ref ops)) => {
        let mut acc = 0;
        for &(ref op, ref _args) in ops {
          acc += self.size_of_op(*op);
        }
        acc
      }
      None => abort!("Unknown opcode: {}", op),
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
      let ret = ::std::mem::replace(
        &mut self.directives[self.idx], Directive::Label(String::new())
      );
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
    fn get_arg(
      this: &Parser, args: &[OpArg], mac_args: &[OpArg], n: usize
    ) -> u16 {
      match args[n] {
        OpArg::Number(n) => n,
        OpArg::Label(ref label) => match this.labels.get(label) {
          Some(&n) => n,
          None => {
            abort!("Use of an undefined label: {}", label);
          }
        },
        OpArg::MacroArg(n) => match mac_args[n as usize] {
          OpArg::Number(n) => n,
          OpArg::Label(ref label) => match this.labels.get(label) {
            Some(&n) => n,
            None => {
              abort!("Use of an undefined label: {}", label);
            }
          },
          OpArg::Here => this.inst_offset,
          OpArg::MacroArg(_) => unreachable!(),
        },
        OpArg::Here => this.inst_offset,
      }
    }
    fn arith(
      this: &Parser,
      op: OpcodeVariant,
      args: &[OpArg],
      mac_args: &[OpArg],
    ) -> (Opcode, u16) {
      let reg = get_arg(this, args, mac_args, 0);
      let num = get_arg(this, args, mac_args, 1);
      if reg >= 0x1000 {
        abort!("Register memory is out of range: {}", reg);
      }
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
      let reg = get_arg(this, &args, mac_args, 0);
      if reg >= 0x1000 {
        abort!("Register memory is out of range: {}", reg);
      }
      let num = get_arg(this, &args, mac_args, 1);
      let label = get_arg(this, &args, mac_args, 2);
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
        data_num.push(match datum {
          OpArg::Number(n) => n,
          OpArg::Label(ref label) => match this.labels.get(label) {
            Some(&n) => n,
            None => abort!("Use of undefined label"),
          },
          OpArg::Here => this.inst_offset,
          OpArg::MacroArg(_) => unreachable!(),
        })
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
    if let Some(directive) = self.next_directive() {
      match directive {
        Directive::Op(op, mac_args) => {
          match self.macros.get(&op) {
            Some(&(ref size, ref ops)) => {
              if (mac_args.len() as u16) != *size {
                abort!(
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
            None => abort!("Unknown opcode"),
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
        Directive::Data(nums) => {
          let (data, offset) = data(self, nums);
          self.inst_offset += offset;
          Some(data)
        }
        Directive::Label(..) | Directive::Const(..) => {
          while let Some(dir) = self.directives.get(self.idx) {
            match *dir {
              Directive::Label(..) | Directive::Const(..) => self.idx += 1,
              _ => break,
            }
          }
          self.next()
        },
        Directive::Macro{..} => unimplemented!(),
      }
    } else {
      None
    }
  }
}
