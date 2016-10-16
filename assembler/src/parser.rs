use std::process;
use std::collections::HashMap;
use {Opcode, OpcodeVariant};

enum Directive {
  Op(String, Vec<Token>),
  Label(String),
  Const(String, Token),
}

pub struct Parser {
  directives: Vec<Directive>,
  labels: HashMap<String, u16>,
  idx: usize,
}

impl Parser {
  pub fn new(input: Vec<u8>) -> Self {
    let mut lexer = Lexer::new(input);
    let mut this = Parser {
      directives: Vec::new(),
      labels: hashmap! {
        "IP".to_owned() => 0x0,
        "SP".to_owned() => 0x1,
        "BP".to_owned() => 0x2,
        "SC".to_owned() => 0x3,
      },
      idx: 0,
    };

    while let Some(tok) = lexer.get_token() {
      match tok {
        Token::Ident(ident) => {
          if ident == "EQU" {
            let lhs = if let Some(tok) = lexer.get_token() {
              match tok {
                Token::Ident(id) => id,
                Token::Label(label) =>
                  abort!("Unexpected colon in const definition: {}", label),
                Token::Number(n) =>
                  abort!("Attempted to define a number: {}", n),
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
          } else if ident == "MACRO" {
            abort!("Reserved identifier: MACRO");
          } else {
            let mut args = Vec::new();
            for _ in 0..this.size_of_op(&ident) {
              match lexer.get_token() {
                // TODO(ubsan): this won't work with arithmetic
                Some(tok) => args.push(tok),
                None => abort!("Unexpected EOF"),
              }
            }
            this.directives.push(Directive::Op(ident, args));
          }
        }
        Token::Number(n) => abort!("Numbers aren't allowed in op position"),
        Token::Label(label) => this.directives.push(Directive::Label(label)),
      }
    }

    // normal labels
    let mut inst_offset = 0x1000;
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
        Directive::Op(ref op, _) => {
          inst_offset += this.size_of_op(op);
        }
        Directive::Const(..) => {}
      }
    }

    // equ constants
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
            Token::Label(ref label) => {
              abort!("Unexpected label definition: {}", label);
            }
          };
          match this.labels.insert(s.clone(), rhs) {
            Some(s) => {
              abort!("Attempted to redefine label: {}", s);
            }
            None => {}
          }
        }
        Directive::Label(..) => {}
        Directive::Op(..) => {}
      }
    }

    this
  }

  // TODO(ubsan): macros
  fn size_of_op(&self, op: &str) -> u16 {
    match op {
      "MI" | "MV" | "MD" | "LD" | "ST" | "AD" | "SB"
      | "ND" | "OR" | "XR" | "SR" | "SL" | "SA" => {
        2
      }
      "JG" | "JL" | "JQ" => {
        3
      }
      s => abort!("Unknown opcode: {}", s),
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
            println!("labels: {:?}", this.labels);
            abort!("Use of an undefined label: {}", id);
          }
        },
        Token::Label(ref label) =>
          abort!("Unexpected label definition: {}", label),
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
    if let Some(directive) = self.next_directive() {
      match directive {
        Directive::Op(s, toks) => {
          match &*s {
            "MI" => arith(self, OpcodeVariant::MoveImmediate, toks),
            "MV" => arith(self, OpcodeVariant::Move, toks),
            "MD" => arith(self, OpcodeVariant::MoveDeref, toks),
            "LD" => arith(self, OpcodeVariant::Load, toks),
            "ST" => arith(self, OpcodeVariant::Store, toks),
            "AD" => arith(self, OpcodeVariant::Add, toks),
            "SB" => arith(self, OpcodeVariant::Sub, toks),
            "ND" => arith(self, OpcodeVariant::And, toks),
            "OR" => arith(self, OpcodeVariant::Or, toks),
            "XR" => arith(self, OpcodeVariant::Xor, toks),
            "SR" => arith(self, OpcodeVariant::ShiftRight, toks),
            "SL" => arith(self, OpcodeVariant::ShiftLeft, toks),
            "SA" => arith(self, OpcodeVariant::ShiftArithmetic, toks),
            "JG" => jump(self, OpcodeVariant::JumpGreater, toks),
            "JL" => jump(self, OpcodeVariant::JumpLesser, toks),
            "JQ" => jump(self, OpcodeVariant::JumpEqual, toks),
            _ => unreachable!(),
          }
        }
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

enum Token {
  Label(String),
  Ident(String),
  Number(u16),
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
      Some(ch) if is_space(ch) || ch == b',' => {
        while let Some(c) = self.peek_char() {
          if is_space(c) { self.get_char(); }
          else { break; }
        }
        self.get_token()
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
      Some(ch) => {
        abort!("Unsupported character: `{}' ({})", ch as char, ch);
      }
      None => None,
    }
  }
}
