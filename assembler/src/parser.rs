use std::process;
use std::collections::HashMap;
use {Opcode, OpcodeVariant};

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

impl BaseOp {
  fn new(s: &str) -> Self {
    match s {
      "MI" => BaseOp::MoveImmediate,
      "MV" => BaseOp::Move,
      "MD" => BaseOp::MoveDeref,
      "LD" => BaseOp::Load,
      "ST" => BaseOp::Store,
      "AD" => BaseOp::Add,
      "SB" => BaseOp::Sub,
      "ND" => BaseOp::And,
      "OR" => BaseOp::Or,
      "XR" => BaseOp::Xor,
      "SR" => BaseOp::ShiftRight,
      "SL" => BaseOp::ShiftLeft,
      "SA" => BaseOp::ShiftArithmetic,
      "JG" => BaseOp::JumpGreater,
      "JL" => BaseOp::JumpLesser,
      "JQ" => BaseOp::JumpEqual,
      s => abort!("Undefined op: {}", s),
    }
  }
}

enum Directive {
  Op(BaseOp, Vec<Token>),
  Data(Vec<Token>),
  Label(String),
  Const(String, Token),
}

pub struct Parser {
  directives: Vec<Directive>,
  labels: HashMap<String, u16>,
  macros: HashMap<String, (u16, Vec<(BaseOp, Vec<Token>)>)>,
  idx: usize,
}

impl Parser {
  pub fn new(input: Vec<u8>) -> Self {
    let mut lexer = Lexer::new(input);
    let mut this = Parser {
      directives: Vec::new(),
      labels: hashmap! {
        "IP".to_owned() => REG_IP,
        "SP".to_owned() => REG_SP,
        "BP".to_owned() => REG_BP,
        "SC".to_owned() => REG_SC,
      },
      macros: hashmap! {
        "HF".to_owned() => (0, vec![
          (BaseOp::JumpEqual, vec![
            Token::Number(REG_IP), Token::Number(REG_IP), Token::Here]),
        ]),
        "JM".to_owned() => (1, vec![
          (BaseOp::Move, vec![
            Token::Number(REG_IP), Token::MacroArg(0)]),
        ]),
        "JI".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            Token::Number(REG_IP), Token::MacroArg(0)]),
        ]),
        "INC".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            Token::Number(REG_SC), Token::Number(1)]),
          (BaseOp::Add, vec![
            Token::MacroArg(0), Token::Number(REG_SC)]),
        ]),
        "DEC".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            Token::Number(REG_SC), Token::Number(1)]),
          (BaseOp::Add, vec![
            Token::MacroArg(0), Token::Number(REG_SC)]),
        ]),
        "NEG".to_owned() => (1, vec![
          (BaseOp::Move, vec![
            Token::Number(REG_SC), Token::MacroArg(0)]),
          (BaseOp::Xor, vec![
            Token::MacroArg(0), Token::MacroArg(0)]),
          (BaseOp::Move, vec![
            Token::MacroArg(0), Token::Number(REG_SC)]),
        ]),
        "ADI".to_owned() => (2, vec![
          (BaseOp::MoveImmediate, vec![
            Token::Number(REG_SC), Token::MacroArg(1)]),
          (BaseOp::Add, vec![
            Token::MacroArg(0), Token::Number(REG_SC)]),
        ]),
        "SBI".to_owned() => (2, vec![
          (BaseOp::MoveImmediate, vec![
            Token::Number(REG_SC), Token::MacroArg(1)]),
          (BaseOp::Sub, vec![
            Token::MacroArg(0), Token::Number(REG_SC)]),
        ]),
        "PUSH".to_owned() => (1, vec![
          (BaseOp::MoveImmediate, vec![
            Token::Number(REG_SC), Token::Number(1)]),
          (BaseOp::Add, vec![
            Token::MacroArg(0), Token::Number(REG_SC)]),
          (BaseOp::Move, vec![
            Token::Number(REG_SP), Token::MacroArg(0)]),
        ]),
        "POP".to_owned() => (1, vec![
          (BaseOp::Move, vec![
            Token::MacroArg(0), Token::Number(REG_SP)]),
          (BaseOp::MoveImmediate, vec![
            Token::Number(REG_SC), Token::Number(1)]),
          (BaseOp::Sub, vec![
            Token::MacroArg(0), Token::Number(REG_SC)]),
        ]),
      },
      idx: 0,
    };

    let mut inst_offset = INST_OFFSET_BASE;
    while let Some(tok) = lexer.get_token() {
      match tok {
        Token::Ident(ref ident) if ident == "EQU" => {
          let lhs = if let Some(tok) = lexer.get_token() {
            match tok {
              Token::Ident(id) => id,
              Token::StringData(_) =>
                abort!("Attempted to define a string"),
              Token::Label(label) =>
                abort!("Unexpected colon in const definition: {}", label),
              Token::Number(n) =>
                abort!("Attempted to define a number: {}", n),
              Token::MacroArg(_) =>
                abort!("Attempted to define a macro argument"),
              Token::Here =>
                abort!("Attempted to define `$'"),
            }
          } else {
            abort!("Unexpected EOF");
          };
          let rhs = if let Some(tok) = lexer.get_token() {
            tok
          } else {
            abort!("Unexpected EOF");
          };
          this.directives.push(Directive::Const(lhs, rhs));
        }
        Token::Ident(ref ident) if ident == "DATA" => {
          let mut data = Vec::new();
          loop {
            match lexer.get_token() {
              Some(Token::Ident(ref s)) if s == "ENDDATA" => break,
              Some(tok @ Token::Ident(_)) | Some(tok @ Token::Number(_)) => {
                inst_offset += 1;
                data.push(tok)
              }
              Some(Token::Here) => {
                inst_offset += 1;
                data.push(Token::Number(inst_offset - 1));
              }
              Some(Token::StringData(s)) => {
                inst_offset += s.len() as u16;
                data.push(Token::StringData(s));
              }
              Some(Token::Label(ref label)) => {
                abort!("Unexpected label definition: {}", label)
              }
              Some(Token::MacroArg(_)) => abort!("Unexpected macro argument"),
              None => abort!("Unexpected EOF (expected ENDDATA)"),
            }
          }
          this.directives.push(Directive::Data(data));
        }
        Token::Ident(ref ident) if ident == "MACRO" => {
          abort!("Reserved identifier: MACRO");
        }
        Token::Ident(ident) => {
          let mut args = Vec::new();
          for _ in 0..this.size_of_op_str(&ident) {
            match lexer.get_token() {
              Some(tok) => args.push(tok),
              None => abort!("Unexpected EOF"),
            }
          }
          if let Some(&(ref size, ref ops)) = this.macros.get(&ident) {
            assert!(*size as usize == args.len());
            for op in ops {
              let mut inner_args = Vec::new();
              for tok in &op.1 {
                if let Token::MacroArg(n) = *tok {
                  inner_args.push(args[n as usize].clone());
                } else if let Token::Here = *tok {
                  inner_args.push(Token::Number(inst_offset));
                } else {
                  inner_args.push(tok.clone());
                }
              }
              inst_offset += this.size_of_op(op.0);
              this.directives.push(Directive::Op(op.0, inner_args));
            }
          } else {
            let op = BaseOp::new(&ident);
            inst_offset += this.size_of_op(op);
            this.directives.push(Directive::Op(op, args));
          }
        }
        Token::Number(_) => abort!("Numbers aren't allowed in op position"),
        Token::Label(label) => this.directives.push(Directive::Label(label)),
        Token::Here => abort!("`$' isn't allowed in op position"),
        Token::MacroArg(_) => abort!("Macro arguments aren't allowed here"),
        Token::StringData(_) =>
          abort!("Strings aren't allowed outside of DATA directives"),
      }
    }

    // normal labels
    let mut inst_offset = INST_OFFSET_BASE;
    for directive in &this.directives {
      match *directive {
        Directive::Label(ref s) => {
          // can optimize this to mem::replace(String::new())
          match this.labels.insert(s.clone(), inst_offset) {
            Some(s) => {
              abort!("Attempted to redefine label: {}", s);
            }
            None => {}
          }
        }
        Directive::Op(op, _) => {
          inst_offset += this.size_of_op(op);
        }
        Directive::Const(..) => {}
        Directive::Data(ref data) => {
          inst_offset += data.len() as u16;
        }
      }
    }

    // equ constants
    let mut inst_offset = INST_OFFSET_BASE;
    for directive in &this.directives {
      match *directive {
        Directive::Const(ref s, ref tok) => {
          let rhs = match *tok {
            Token::Ident(ref ident) => {
              if let Some(&n) = this.labels.get(ident) {
                n
              } else {
                abort!(
                  "Attempted to use undefined label in a constant definition: {}",
                  ident,
                );
              }
            }
            Token::Number(n) => n,
            Token::Here => inst_offset,
            Token::StringData(_)
              => abort!("Attempted to define a constant to a string"),
            Token::Label(ref label)
              => abort!("Unexpected label definition: {}", label),
            Token::MacroArg(_)
              => abort!("Unexpected macro argument in a constant definition"),
          };
          match this.labels.insert(s.clone(), rhs) {
            Some(s) => {
              abort!("Attempted to redefine label: {}", s);
            }
            None => {}
          }
        }
        Directive::Label(..) => {}
        Directive::Op(op, _) => {
          inst_offset += this.size_of_op(op);
        }
        Directive::Data(ref data) => {
          inst_offset += data.len() as u16;
        }
      }
    }

    this
  }

  fn size_of_op_str(&self, op: &str) -> u16 {
    match op {
      "MI" | "MV" | "MD" | "LD" | "ST" | "AD" | "SB"
      | "ND" | "OR" | "XR" | "SR" | "SL" | "SA" => {
        2
      }
      "JG" | "JL" | "JQ" => {
        3
      }
      s => {
        if let Some(&(size, _)) = self.macros.get(s) {
          size
        } else {
          abort!("Unknown opcode: {}", s);
        }
      }
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
    fn get_arg(this: &Parser, args: &[Token], n: usize) -> u16 {
      match args[n] {
        Token::Number(n) => n,
        Token::Ident(ref id) => match this.labels.get(id) {
          Some(&n) => n,
          None => {
            abort!("Use of an undefined label: {}", id);
          }
        },
        Token::StringData(_) => unreachable!(),
        Token::Label(_) => unreachable!(),
        Token::Here => unreachable!(),
        Token::MacroArg(_) => unreachable!(),
      }
    }
    fn arith(
      this: &Parser, op: OpcodeVariant, args: Vec<Token>
    ) -> Option<Opcode> {
      let reg = get_arg(this, &args, 0);
      let num = get_arg(this, &args, 1);
      if reg >= 0x1000 {
        abort!("Register memory is out of range: {}", reg);
      }
      Some(Opcode {
        variant: op,
        reg: reg,
        num: num,
      })
    }
    fn jump(
      this: &Parser, op: fn(u16) -> OpcodeVariant, args: Vec<Token>
    ) -> Option<Opcode> {
      let reg = get_arg(this, &args, 0);
      if reg >= 0x1000 {
        abort!("Register memory is out of range: {}", reg);
      }
      let num = get_arg(this, &args, 1);
      let label = get_arg(this, &args, 2);
      Some(Opcode {
        variant: op(label),
        reg: reg,
        num: num,
      })
    }
    fn data(this: &Parser, data: Vec<Token>) -> Option<Opcode> {
      let mut data_num = Vec::new();
      // heh. datum.
      for datum in data {
        match datum {
          Token::Number(n) => data_num.push(n),
          Token::Ident(ref id) => match this.labels.get(id) {
            Some(&n) => data_num.push(n),
            None => {
              abort!("Use of an undefined label: {}", id);
            }
          },
          Token::StringData(mut s) => data_num.append(&mut s),
          Token::Label(_) => unreachable!(),
          Token::Here => unreachable!(),
          Token::MacroArg(_) => unreachable!(),
        };
      }
      Some(Opcode {
        variant: OpcodeVariant::Data(data_num),
        reg: 0,
        num: 0,
      })
    }
    if let Some(directive) = self.next_directive() {
      match directive {
        Directive::Op(op, toks) => {
          match op {
            BaseOp::MoveImmediate =>
              arith(self, OpcodeVariant::MoveImmediate, toks),
            BaseOp::Move => arith(self, OpcodeVariant::Move, toks),
            BaseOp::MoveDeref => arith(self, OpcodeVariant::MoveDeref, toks),
            BaseOp::Load => arith(self, OpcodeVariant::Load, toks),
            BaseOp::Store => arith(self, OpcodeVariant::Store, toks),
            BaseOp::Add => arith(self, OpcodeVariant::Add, toks),
            BaseOp::Sub => arith(self, OpcodeVariant::Sub, toks),
            BaseOp::And => arith(self, OpcodeVariant::And, toks),
            BaseOp::Or => arith(self, OpcodeVariant::Or, toks),
            BaseOp::Xor => arith(self, OpcodeVariant::Xor, toks),
            BaseOp::ShiftRight => arith(self, OpcodeVariant::ShiftRight, toks),
            BaseOp::ShiftLeft => arith(self, OpcodeVariant::ShiftLeft, toks),
            BaseOp::ShiftArithmetic =>
              arith(self, OpcodeVariant::ShiftArithmetic, toks),
            BaseOp::JumpGreater => jump(self, OpcodeVariant::JumpGreater, toks),
            BaseOp::JumpLesser => jump(self, OpcodeVariant::JumpLesser, toks),
            BaseOp::JumpEqual => jump(self, OpcodeVariant::JumpEqual, toks),
          }
        }
        Directive::Data(nums) => data(self, nums),
        Directive::Label(..) | Directive::Const(..) => {
          while let Some(dir) = self.directives.get(self.idx) {
            match *dir {
              Directive::Label(..) | Directive::Const(..) => self.idx += 1,
              _ => break,
            }
          }
          self.next()
        }
      }
    } else {
      None
    }
  }
}

#[derive(Clone)]
enum Token {
  Label(String),
  Ident(String),
  MacroArg(u16),
  Number(u16),
  StringData(Vec<u16>),
  Here, // $
}

struct Lexer {
  input: Vec<u8>,
  idx: usize,
}
impl Lexer {
  fn new(input: Vec<u8>) -> Self {
    Lexer {
      input: input,
      idx: 0
    }
  }

  fn peek_char(&self) -> Option<u8> {
    fn toupper(c: u8) -> u8 {
      if c >= b'a' && c <= b'z' {
        c - (b'a' - b'A')
      } else {
        c
      }
    }
    self.input.get(self.idx).map(|&c| toupper(c))
  }

  fn get_char(&mut self) -> Option<u8> {
    match self.peek_char() {
      Some(c) => {
        self.idx += 1;
        Some(c)
      }
      None => None,
    }
  }

  fn get_token(&mut self) -> Option<Token> {
    fn is_space(c: u8) -> bool {
      c == b' ' || c == b'\t' || c == b'\n' ||
        c == 0x0b || c == 0x0c  || c == b'\r'
    }
    fn is_alpha(c: u8) -> bool {
      (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z')
    }
    fn is_ident_start(c: u8) -> bool {
      is_alpha(c) || c == b'_'
    }
    fn is_num(c: u8) -> bool {
      c >= b'0' && c <= b'9'
    }
    fn is_ident(c: u8) -> bool {
      is_ident_start(c) || is_num(c)
    }
    fn is_allowed(c: u8, base: u32) -> bool {
      match base {
        2 => c >= b'0' && c < b'2',
        8 => c >= b'0' && c < b'8',
        10 => is_num(c),
        16 => is_num(c) || (c >= b'A' && c <= b'F'),
        _ => process::exit(192),
      }
    }
    match self.get_char() {
      Some(ch) if ch == b'#' || ch == b';' => {
        while let Some(c) = self.peek_char() {
          if c != b'\n' { self.get_char(); }
          else { break; }
        }
        self.get_token()
      }
      Some(ch) if is_space(ch) || ch == b',' => {
        while let Some(c) = self.peek_char() {
          if is_space(c) { self.get_char(); }
          else { break; }
        }
        self.get_token()
      }
      Some(ch) if ch == b'"' || ch == b'\'' => {
        let mut data = Vec::new();
        loop {
          match self.get_char() {
            Some(c) if c == ch => break,
            Some(b'\\') => {
              match self.get_char() {
                Some(b'a') => data.push(0x07),  // alarm
                Some(b'b') => data.push(0x08),  // backspace
                Some(b'f') => data.push(0x0C),  // Formfeed
                Some(b'n') => data.push(0x0A),  // Line Feed
                Some(b'r') => data.push(0x0D),  // Carriage Return
                Some(b't') => data.push(0x09),  // Horizontal Tab
                Some(b'v') => data.push(0x0B),  // Vertical Tab
                Some(b'\\') => data.push(0x5C), // Backslash
                Some(b'\'') => data.push(0x27), // Single quotation mark
                Some(b'\"') => data.push(0x22), // Double quotation mark
                Some(b'?') => data.push(0x3F),  // Question mark
                Some(b'\n') => {}
                Some(c) =>
                  abort!("Unknown character after \\: `{}' ({})", c as char, c),
                None => abort!("Unexpected EOF"),
              }
            }
            Some(c) => {
              data.push(c as u16)
            }
            None => abort!("Unexpected EOF"),
          }
        }
        Some(Token::StringData(data))
      }
      Some(ch) if is_ident_start(ch) => {
        let mut ret = Vec::new();
        ret.push(ch);
        while let Some(c) = self.peek_char() {
          if is_ident(c) {
            self.get_char();
            ret.push(c);
          } else if c == b':' {
            self.get_char();
            return Some(Token::Label(String::from_utf8(ret).unwrap()));
          } else {
            break;
          }
        }
        Some(Token::Ident(String::from_utf8(ret).unwrap()))
      }
      Some(ch) if is_num(ch) => {
        let mut base = 10;
        let mut ret = Vec::new();
        if ch == b'0' {
          let peek = self.peek_char().unwrap_or(b'D');
          if is_space(peek) || peek == b',' {
            return Some(Token::Number(0));
          } else if is_num(peek) {
            while let Some(ch) = self.peek_char() {
              if ch == b'0' { self.get_char(); }
              else { break; }
            }
          } else if peek == b'B' {
            base = 2;
            self.get_char();
          } else if peek == b'O' {
            base = 8;
            self.get_char();
          } else if peek == b'D' {
            base = 10;
            self.get_char();
          } else if peek == b'X' {
            base = 16;
            self.get_char();
          } else {
            abort!(
              "Unsupported character in a number literal: {}", peek as char
            );
          }
        }

        while let Some(ch) = self.peek_char() {
          if is_allowed(ch, base as u32) {
            self.get_char();
            ret.push(ch);
          } else if is_alpha(ch) {
            abort!(
              "Unsupported character in a base-{} literal: {}",
              base,
              ch as char,
            );
          } else {
            break;
          }
        }
        let mut acc = 0u16;
        for el in &ret {
          let add = if is_alpha(*el) {
            el - b'A' + 10
          } else {
            el - b'0'
          } as u16;
          acc = match acc.checked_mul(base).and_then(|a| a.checked_add(add)) {
              Some(a) => a,
              None => {
                abort!(
                  "Attempted to write an overflowing number literal: {}",
                  unsafe{::std::str::from_utf8_unchecked(&ret)},
                );
              }
            }
        }
        Some(Token::Number(acc))
      }
      Some(b'$') => Some(Token::Here),
      Some(ch) => {
        abort!("Unsupported character: `{}' ({})", ch as char, ch);
      }
      None => None,
    }
  }
}
