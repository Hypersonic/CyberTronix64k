use std::collections::HashMap;

#[derive(Clone)]
pub struct OpArg {
  pub variant: OpArgVar,
  pub line: usize,
  pub offset: usize,
}

impl OpArg {
  pub fn evaluate(
    &self, labels: &HashMap<String, u16>, mac_args: &[OpArg], inst_offset: u16,
  ) -> u16 {
    match self.variant {
      OpArgVar::Number(n) => n,
      OpArgVar::Label(ref label) => match labels.get(label) {
        Some(&n) => n,
        None => error!(self.line, self.offset, "Undefined label: {}", label),
      },
      OpArgVar::MacroArg(n) =>
        mac_args[n as usize].evaluate(labels, &[], inst_offset),
      OpArgVar::ArithOp(ref op, ref lhs, ref rhs) => op.op(
        lhs.evaluate(labels, mac_args, inst_offset),
        rhs.evaluate(labels, mac_args, inst_offset),
      ),
      OpArgVar::Here => inst_offset,
    }
  }
}

#[derive(Copy, Clone)]
pub enum ArithOp {
  Add,
  #[allow(dead_code)]
  Sub,
  #[allow(dead_code)]
  Mul,
  #[allow(dead_code)]
  Div,
}

impl ArithOp {
  pub fn op(self, lhs: u16, rhs: u16) -> u16 {
    match self {
      ArithOp::Add => lhs + rhs,
      ArithOp::Sub => lhs - rhs,
      ArithOp::Mul => lhs * rhs,
      ArithOp::Div => lhs / rhs,
    }
  }
}

#[derive(Clone)]
pub enum OpArgVar {
  Number(u16),
  Label(String),
  MacroArg(u16),
  ArithOp(ArithOp, Box<OpArg>, Box<OpArg>),
  Here, // $
}

#[derive(Clone)]
pub enum DirectiveVar {
  Label(String),
  Op(String, Vec<OpArg>),
  Const(String, OpArg),
  // TODO(ubsan): allow non-constant reps?
  Data(Vec<OpArg>),
  #[allow(dead_code)]
  Macro {
    name: String,
    args: u16,
    expansions: Vec<(String, Vec<OpArg>)>,
  },
}

#[derive(Clone)]
pub struct Directive {
  pub variant: DirectiveVar,
  pub line: usize,
  pub offset: usize,
}

enum TokenVar {
  Ident(Vec<u8>),
  Label(Vec<u8>),
  StrLit(Vec<u8>),
  NumLit(u16),
  MacroArg(u16),
  MacroLabel(Vec<u8>),
  Data,
  Equ,
  Rep,
  Macro,
  EndMacro,
  Here, // $
  Comma,
  Newline,
}

struct Token {
  variant: TokenVar,
  line: usize,
  offset: usize,
}

pub struct Lexer {
  input: Vec<u8>,
  idx: usize,
  line: usize,
  offset: usize,
}
impl Lexer {
  pub fn new(input: Vec<u8>) -> Self {
    Lexer {
      input: input,
      idx: 0,
      line: 1,
      offset: 0,
    }
  }

  pub fn next_directive(&mut self) -> Option<Directive> {
    fn to_string(line: usize, offset: usize, v: Vec<u8>) -> String {
      match String::from_utf8(v) {
        Ok(s) => s,
        Err(_) => error!(line, offset, "Invalid utf8"),
      }
    }
    // None means EOL
    fn get_op_arg(tok: Token) -> Option<OpArg> {
      match tok.variant {
        TokenVar::Newline => None,
        TokenVar::Here => Some(OpArg {
          variant: OpArgVar::Here,
          line: tok.line,
          offset: tok.offset,
        }),
        TokenVar::Ident(id) => Some(OpArg {
          variant: OpArgVar::Label(to_string(tok.line, tok.offset, id)),
          line: tok.line,
          offset: tok.offset,
        }),
        TokenVar::NumLit(n) => Some(OpArg {
          variant: OpArgVar::Number(n),
          line: tok.line,
          offset: tok.offset,
        }),
        TokenVar::StrLit(s) => {
          if s.len() == 1 {
            Some(OpArg {
              variant: OpArgVar::Number(s[0] as u16),
              line: tok.line,
              offset: tok.offset,
            })
          } else if s.is_empty() {
            error!(
              tok.line, tok.offset, "Unexpected empty string literal",
            );
          } else {
            error!(
              tok.line, tok.offset, "Unexpected multi-char string literal",
            );
          }
        }
        TokenVar::MacroLabel(_) => error!(
          tok.line,
          tok.offset,
          "Unexpected macro label outside macro context",
        ),
        TokenVar::MacroArg(_) => error!(
          tok.line,
          tok.offset,
          "Unexpected macro argument outside macro context",
        ),
        TokenVar::Macro => error!(
          tok.line, tok.offset, "Unexpected macro start",
        ),
        TokenVar::EndMacro => error!(
          tok.line, tok.offset, "Unexpected macro end",
        ),
        TokenVar::Equ => error!(
          tok.line, tok.offset, "Unexpected EQU directive",
        ),
        TokenVar::Rep => error!(
          tok.line, tok.offset, "Unexpected REP directive",
        ),
        TokenVar::Data => error!(
          tok.line, tok.offset, "Unexpected DATA directive",
        ),
        TokenVar::Comma => error!(
          tok.line, tok.offset, "Unexpected comma",
        ),
        TokenVar::Label(_) => error!(
          tok.line, tok.offset, "Unexpected label",
        ),
      }
    }
    if let Some(tok) = self.next_token() {
      match tok.variant {
        TokenVar::Newline => self.next_directive(),
        TokenVar::Label(label) => Some(Directive {
          variant: DirectiveVar::Label(to_string(tok.line, tok.offset, label)),
          line: tok.line,
          offset: tok.offset,
        }),
        TokenVar::Ident(op) => {
          let line = tok.line;
          let offset = tok.offset;
          let op = to_string(tok.line, tok.offset, op);
          let mut args = Vec::new();
          while let Some(tok) = self.next_token() {
            if let Some(arg) = get_op_arg(tok) {
              args.push(arg);
            }
            if let Some(tok) = self.next_token() {
              if let TokenVar::Newline = tok.variant {
                break;
              } else if let TokenVar::Comma = tok.variant {
              } else {
                error!(tok.line, tok.offset, "Expected a comma or a newline");
              }
            }
          }
          Some(Directive {
            variant: DirectiveVar::Op(op, args),
            line: line,
            offset: offset,
          })
        },
        TokenVar::Data => {
          let line = tok.line;
          let offset = tok.offset;
          let mut data = Vec::new();
          while let Some(tok) = self.next_token() {
            match tok.variant {
              TokenVar::Rep => {
                let repetitions = match self.next_token() {
                  Some(tok) => match tok.variant {
                    TokenVar::NumLit(n) => n,
                    _ => error!(
                      tok.line,
                      tok.offset,
                      "Expected literal number of repetitions",
                    ),
                  },
                  None => error!(tok.line, tok.offset, "Unexpected EOF"),
                };
                match self.next_token() {
                  Some(tok) => match get_op_arg(tok) {
                    Some(op) => for _ in 0..repetitions {
                      data.push(op.clone());
                    },
                    None =>
                      error!(self.line, self.offset, "Unexpected newline"),
                  },
                  None => error!(tok.line, tok.offset, "Unexpected EOF"),
                }
              }
              TokenVar::Ident(id) => data.push(OpArg {
                variant: OpArgVar::Label(to_string(tok.line, tok.offset, id)),
                line: tok.line,
                offset: tok.offset,
              }),
              TokenVar::StrLit(s) => {
                let line = tok.line;
                let offset = tok.offset;
                data.extend(s.into_iter().map(|c| OpArg {
                  variant: OpArgVar::Number(c as u16),
                  line: line,
                  offset: offset,
                }))
              }
              TokenVar::NumLit(n) => data.push(OpArg {
                variant: OpArgVar::Number(n),
                line: tok.line,
                offset: tok.offset,
              }),
              TokenVar::Here => data.push(OpArg {
                variant: OpArgVar::Here,
                line: tok.line,
                offset: tok.offset,
              }),
              TokenVar::Newline => break,
              TokenVar::Comma => error!(tok.line, tok.offset, "Unexpected comma"),
              TokenVar::Label(_) =>
                error!(tok.line, tok.offset, "Unexpected label definition"),
              TokenVar::MacroLabel(_) => error!(
                tok.line,
                tok.offset,
                "Unexpected macro label outside of macro context"
              ),
              TokenVar::MacroArg(_) => error!(
                tok.line,
                tok.offset,
                "Unexpected macro argument outside of macro context",
              ),
              TokenVar::Data =>
                error!(tok.line, tok.offset, "Unexpected DATA directive"),
              TokenVar::Equ =>
                error!(tok.line, tok.offset, "Unexpected EQU directive"),
              TokenVar::Macro =>
                error!(tok.line, tok.offset, "Unexpected MACRO directive"),
              TokenVar::EndMacro =>
                error!(tok.line, tok.offset, "Unexpected ENDMACRO directive"),
            }
          }
          Some(Directive {
            variant: DirectiveVar::Data(data),
            line: line,
            offset: offset,
          })
        }
        TokenVar::Equ => {
          let line = tok.line;
          let offset = tok.offset;
          let name = if let Some(tok) = self.next_token() {
            match tok.variant {
              TokenVar::Ident(s) => to_string(tok.line, tok.offset, s),
              _ => error!(
                tok.line, tok.offset, "Expected identifier for EQU directive",
              ),
            }
          } else {
              error!(tok.line, tok.offset, "Unexpected EOF")
          };
          let constant = if let Some(tok) = self.next_token() {
            match get_op_arg(tok) {
              Some(op) => op,
              None => error!(self.line, self.offset, "Unexpected newline"),
            }
          } else {
            error!(tok.line, tok.offset, "Unexpected EOF")
          };
          Some(Directive {
            variant: DirectiveVar::Const(name, constant),
            line: line,
            offset: offset,
          })
        }
        TokenVar::Rep =>
          error!(tok.line, tok.offset, "Unexpected REP directive"),
        TokenVar::Comma => error!(tok.line, tok.offset, "Unexpected comma"),
        TokenVar::Here => error!(tok.line, tok.offset, "Unexpected $"),
        TokenVar::StrLit(_) =>
          error!(tok.line, tok.offset, "Unexpected string literal"),
        TokenVar::NumLit(_) =>
          error!(tok.line, tok.offset, "Unexpected number literal"),
        TokenVar::MacroLabel(_) =>
          error!(tok.line, tok.offset, "Unexpected macro label"),
        TokenVar::MacroArg(_) =>
          error!(tok.line, tok.offset, "Unexpected macro argument"),
        TokenVar::Macro | TokenVar::EndMacro =>
          error!(tok.line, tok.offset, "Macros are not yet implemented"),
      }
    } else {
      None
    }
  }

  fn peek_char(&self) -> Option<u8> {
    self.input.get(self.idx).cloned()
  }

  fn get_char(&mut self) -> Option<u8> {
    match self.peek_char() {
      Some(c) => {
        self.idx += 1;
        if c == b'\n' {
          self.offset = 0;
          self.line += 1;
        } else {
          self.offset += 1;
        }
        Some(c)
      }
      None => None,
    }
  }

  fn next_token(&mut self) -> Option<Token> {
    fn is_space(c: u8) -> bool {
      c == b' ' || c == b'\t' || c == 0x0b || c == 0x0c  || c == b'\r'
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
    fn to_upper(c: u8) -> u8 {
      if c >= b'a' && c <= b'z' {
        c - (b'a' - b'A')
      } else {
        c
      }
    }
    fn is_allowed(c: u8, base: u16) -> bool {
      match base {
        2 => c >= b'0' && c < b'2',
        8 => c >= b'0' && c < b'8',
        10 => is_num(c),
        16 => is_num(c) || (c >= b'A' && c <= b'F'),
        _ => unreachable!(),
      }
    }

    match self.get_char() {
      Some(b'\\') => {
        let ch = self.get_char();
        if let Some(b'\n') = ch {
          self.next_token()
        } else if let Some(ch) = ch {
          error!(self.line, self.offset, "Unexpected `{}' ({})", ch as char, ch)
        } else {
          error!(self.line, self.offset, "Unexpected EOF")
        }
      }
      Some(ch) if ch == b'#' || ch == b';' => {
        while let Some(c) = self.peek_char() {
          if c != b'\n' { self.get_char(); }
          else { break; }
        }
        self.next_token()
      }
      Some(ch) if is_space(ch) => {
        while let Some(c) = self.peek_char() {
          if is_space(c) { self.get_char(); }
          else { break; }
        }
        self.next_token()
      }
      Some(b'\n') => Some(Token {
        variant: TokenVar::Newline,
        line: self.line,
        offset: self.offset,
      }),
      Some(b',') => Some(Token {
        variant: TokenVar::Comma,
        line: self.line,
        offset: self.offset,
      }),
      Some(b'$') => Some(Token {
        variant: TokenVar::Here,
        line: self.line,
        offset: self.offset,
      }),
      Some(quote) if quote == b'\'' || quote == b'"' => {
        let line = self.line;
        let offset = self.offset;
        let mut buff = Vec::new();
        while let Some(ch) = self.get_char() {
          if ch == b'\\' {
            match self.get_char() {
              Some(b'\'') => buff.push(b'\''),
              Some(b'"') => buff.push(b'"'),
              Some(b'\n') => {
                loop {
                  if let Some(c) = self.peek_char() {
                    if is_space(c) {
                      self.get_char();
                    } else {
                      break;
                    }
                  } else {
                    error!(self.line, self.offset, "Unexpected EOF")
                  }
                }
              },
              Some(b'n') => buff.push(b'\n'),
              Some(ch) => error!(
                self.line,
                self.offset,
                "Unrecogized escape sequence: \\{}",
                ch
              ),
              None => error!(self.line, self.offset, "Unexpected EOF"),
            }
          } else if ch == quote {
            break;
          } else {
            buff.push(ch);
          }
        }
        Some(Token {
          variant: TokenVar::StrLit(buff),
          line: line,
          offset: offset,
        })
      }
      Some(ch) if is_ident_start(ch) => {
        let line = self.line;
        let offset = self.offset;
        let mut ret = Vec::new();
        ret.push(to_upper(ch));
        while let Some(c) = self.peek_char() {
          if is_ident(c) {
            self.get_char();
            ret.push(to_upper(c));
          } else if c == b':' {
            self.get_char();
            return Some(Token {
              variant: TokenVar::Label(ret),
              line: line,
              offset: offset,
            });
          } else {
            break;
          }
        }
        Some(Token {
          variant: {
            if ret == b"DATA" {
              TokenVar::Data
            } else if ret == b"EQU" {
              TokenVar::Equ
            } else if ret == b"REP" {
              TokenVar::Rep
            } else if ret == b"MACRO" {
              TokenVar::Macro
            } else if ret == b"ENDMACRO" {
              TokenVar::EndMacro
            } else {
              TokenVar::Ident(ret)
            }
          },
          line: line,
          offset: offset,
        })
      }
      Some(ch) if is_num(ch) => {
        let line = self.line;
        let offset = self.offset;
        let mut base = 10;
        let mut ret = Vec::new();
        if ch == b'0' {
          match self.peek_char().map(to_upper) {
            Some(b'B') => base = 2,
            Some(b'O') => base = 8,
            Some(b'D') => base = 10,
            Some(b'X') => base = 16,
            Some(ch) if is_num(ch) => ret.push(ch),
            Some(ch) if is_alpha(ch) => error!(
              self.line,
              self.offset,
              "Unsupported character in a base-{} literal: {}",
              base,
              ch as char,
            ),
            Some(_) | None => return Some(Token {
              variant: TokenVar::NumLit(0),
              line: line,
              offset: offset,
            })
          }
          self.get_char();
        }

        while let Some(ch) = self.peek_char().map(to_upper) {
          if is_allowed(ch, base) {
            self.get_char();
            ret.push(ch);
          } else if is_alpha(ch) {
            error!(
              self.line,
              self.offset,
              "Unsupported character in a base-{} literal: {}",
              base,
              ch as char,
            );
          } else {
            break;
          }
        }
        let ret = ret.iter().fold(0, |acc: u16, &el: &u8| {
          let add = if is_alpha(el) { el - b'A' + 10 } else { el - b'0' };
          match acc.checked_mul(base).and_then(|a| a.checked_add(add as u16)) {
            Some(a) => a,
            None => error!(
              self.line,
              self.offset,
              "Attempted to write an overflowing number literal: {}",
              ::std::str::from_utf8(&ret).unwrap(),
            ),
          }
        });
        Some(Token {
          variant: TokenVar::NumLit(ret),
          line: line,
          offset: offset,
        })
      }
      Some(b'%') => {
        let line = self.line;
        let offset = self.offset;
        if let Some(next_tok) = self.next_token() {
          if let TokenVar::NumLit(n) = next_tok.variant {
            Some(Token {
              variant: TokenVar::MacroArg(n),
              line: line,
              offset: offset,
            })
          } else if let TokenVar::Label(label) = next_tok.variant {
            Some(Token {
              variant: TokenVar::MacroLabel(label),
              line: line,
              offset: offset,
            })
          } else if let TokenVar::Ident(_) = next_tok.variant {
            error!(self.line, self.offset, "Expected a colon");
          } else {
            error!(
              next_tok.line,
              next_tok.offset,
              "Expected a label or number"
            );
          }
        } else {
          error!(self.line, self.offset, "Unexpected EOF");
        }
      }
      Some(ch) => error!(
        self.line,
        self.offset,
        "Unsupported character: `{}' ({})",
        ch as char,
        ch
      ),
      None => None,
    }
  }
}
