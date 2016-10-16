use std::process;
use {Opcode, OpcodeVariant};

pub enum OpOrLabel {
  Op(Opcode),
  Label(String, u16),
}

pub struct Parser {
  lexer: Lexer,
  offset: u16,
}

impl Parser {
  pub fn new(input: Vec<u8>) -> Self {
    Parser { lexer: Lexer::new(input), offset: 0 }
  }

  fn get_rm(&mut self) -> ::Number {
    let ret = self.get_num();
    match ret {
      ::Number::Immediate(n) if n >= 0x1000 => {
        eprintln!("Overflowing literal for register memory: 0x{:x}", n);
        process::exit(1);
      }
      _ => {}
    }
    ret
  }
  fn get_num(&mut self) -> ::Number {
    use self::Token::*;
    match self.lexer.get_token() {
      Some(Ident(ident)) => ::Number::Label(ident),
      Some(Number(n)) => {
        ::Number::Immediate(n)
      }
      Some(Label(label)) => {
        eprintln!("Did not expect label definition: {}", label);
        process::exit(1);
      }
      None => {
        eprintln!("Unexpected EOF");
        process::exit(1);
      }
    }
  }
}

impl Iterator for Parser {
  type Item = OpOrLabel;

  fn next(&mut self) -> Option<OpOrLabel> {
    use self::Token::*;
    fn arith(this: &mut Parser, variant: OpcodeVariant) -> Opcode {
      Opcode {
        variant: variant,
        reg: this.get_rm(),
        mem: this.get_num(),
      }
    }
    fn jump(
      this: &mut Parser, variant: fn(::Number) -> OpcodeVariant
    ) -> Opcode {
      Opcode {
        reg: this.get_rm(),
        mem: this.get_num(),
        variant: variant(this.get_num()),
      }
    }
    match self.lexer.get_token() {
      Some(Label(label)) => {
        Some(OpOrLabel::Label(label, self.offset))
      }
      Some(Ident(ident)) => {
        Some(OpOrLabel::Op(
          match &*ident {
            "MI" => arith(self, OpcodeVariant::MoveImmediate),
            "MV" => arith(self, OpcodeVariant::Move),
            "MD" => arith(self, OpcodeVariant::MoveDeref),
            "LD" => arith(self, OpcodeVariant::Load),
            "ST" => arith(self, OpcodeVariant::Store),
            "AD" => arith(self, OpcodeVariant::Add),
            "SB" => arith(self, OpcodeVariant::Sub),
            "ND" => arith(self, OpcodeVariant::And),
            "OR" => arith(self, OpcodeVariant::Or),
            "XR" => arith(self, OpcodeVariant::Xor),
            "SR" => arith(self, OpcodeVariant::ShiftRight),
            "SL" => arith(self, OpcodeVariant::ShiftLeft),
            "SA" => arith(self, OpcodeVariant::ShiftArithmetic),
            "JG" => jump(self, OpcodeVariant::JumpGreater),
            "JL" => jump(self, OpcodeVariant::JumpLesser),
            "JQ" => jump(self, OpcodeVariant::JumpEqual),
            "HF" => Opcode {
              variant: OpcodeVariant::JumpEqual(::Number::Label(String::new())),
              reg: ::Number::Immediate(0),
              mem: ::Number::Immediate(0),
            },
            op => {
              eprintln!("Unsupported op code: {}", op);
              process::exit(1);
            }
          }
        ))
      }
      Some(Number(n)) => {
        eprintln!("Unexpected number literal: 0x{:x}", n);
        process::exit(1);
      }
      None => None,
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
    fn is_num(c: u8) -> bool {
      c >= b'0' && c <= b'9'
    }
    fn is_alnum(c: u8) -> bool {
      is_alpha(c) || is_num(c)
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
      Some(ch) if is_alpha(ch) => {
        let mut ret = Vec::new();
        ret.push(ch);
        while let Some(c) = self.peek_char() {
          if is_alnum(c) {
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
          if is_num(peek) {
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
            eprintln!(
              "Unsupported character in a number literal: {}", peek as char
            );
            process::exit(1);
          }
        }

        while let Some(ch) = self.peek_char() {
          if is_allowed(ch, base as u32) {
            self.get_char();
            ret.push(ch);
          } else if is_alpha(ch) {
            eprintln!(
              "Unsupported character in a number literal: {}", ch as char
            );
            process::exit(1);
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
                eprintln!(
                  "Attempted to write an overflowing number literal: {}",
                  unsafe{::std::str::from_utf8_unchecked(&ret)},
                );
                process::exit(1);
              }
            }
        }
        Some(Token::Number(acc))
      }
      Some(ch) => {
        eprintln!("Unsupported character: `{}' ({})", ch as char, ch);
        process::exit(1);
      }
      None => None,
    }
  }
}
