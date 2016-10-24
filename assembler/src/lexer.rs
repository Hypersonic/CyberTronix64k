#[derive(Clone)]
pub enum OpArg {
  Number(u16),
  Label(String),
  MacroArg(u16),
  Here, // $
}

#[derive(Clone)]
pub enum Directive {
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

enum Token {
  Ident(Vec<u8>),
  Label(Vec<u8>),
  StrLit(Vec<u8>),
  NumLit(u16),
  MacroArg(u16),
  Data,
  Equ,
  Rep,
  Macro,
  EndMacro,
  Here, // $
  Comma,
  Newline,
}

pub struct Lexer {
  input: Vec<u8>,
  idx: usize,
}
impl Lexer {
  pub fn new(input: Vec<u8>) -> Self {
    Lexer {
      input: input,
      idx: 0
    }
  }

  pub fn next_directive(&mut self) -> Option<Directive> {
    fn to_string(v: Vec<u8>) -> String {
      match String::from_utf8(v) {
        Ok(s) => s,
        Err(_) => abort!("Invalid utf8"),
      }
    }
    // None means EOL
    fn get_op_arg(tok: Token) -> Option<OpArg> {
      match tok {
        Token::Newline => None,
        Token::Here => Some(OpArg::Here),
        Token::Ident(id) => Some(OpArg::Label(to_string(id))),
        Token::NumLit(n) => Some(OpArg::Number(n)),
        Token::StrLit(s) => {
          if s.len() == 1 {
            Some(OpArg::Number(s[0] as u16))
          } else if s.is_empty() {
            abort!("Unexpected empty string literal");
          } else {
            abort!("Unexpected multi-char string literal");
          }
        }
        Token::MacroArg(_) =>
          abort!("Unexpected macro argument outside macro context"),
        Token::Macro => abort!("Unexpected macro start"),
        Token::EndMacro => abort!("Unexpected macro end"),
        Token::Equ => abort!("Unexpected EQU directive"),
        Token::Rep => abort!("Unexpected REP directive"),
        Token::Data => abort!("Unexpected DATA directive"),
        Token::Comma => abort!("Unexpected comma"),
        Token::Label(_) => abort!("Unexpected label"),
      }
    }
    match self.next_token() {
      Some(Token::Newline) => self.next_directive(),
      Some(Token::Label(label)) => Some(Directive::Label(to_string(label))),
      Some(Token::Ident(op)) => {
        let op = to_string(op);
        let mut args = Vec::new();
        while let Some(tok) = self.next_token() {
          if let Some(arg) = get_op_arg(tok) {
            args.push(arg);
          }
          if let Some(tok) = self.next_token() {
            if let Token::Newline = tok {
              break;
            } else if let Token::Comma = tok {
            } else {
              abort!("Expected a comma or a newline");
            }
          }
        }
        Some(Directive::Op(op, args))
      },
      Some(Token::Data) => {
        let mut data = Vec::new();
        while let Some(tok) = self.next_token() {
          match tok {
            Token::Rep => {
              let repetitions = match self.next_token() {
                Some(tok) => match tok {
                  Token::NumLit(n) => n,
                  _ => abort!("Expected literal number of repetitions"),
                },
                None => abort!("Unexpected EOF"),
              };
              match self.next_token() {
                Some(tok) => match get_op_arg(tok) {
                  Some(op) => for _ in 0..repetitions {
                    data.push(op.clone());
                  },
                  None => abort!("Unexpected newline"),
                },
                None => abort!("Unexpected EOF"),
              }
            }
            Token::Ident(id) => data.push(OpArg::Label(to_string(id))),
            Token::StrLit(s) =>
              data.extend(s.into_iter().map(|c| OpArg::Number(c as u16))),
            Token::NumLit(n) => data.push(OpArg::Number(n)),
            Token::Here => data.push(OpArg::Here),
            Token::Newline => break,
            Token::Comma => abort!("Unexpected comma"),
            Token::Label(_) => abort!("Unexpected label definition"),
            Token::MacroArg(_) =>
              abort!("Unexpected macro argument outside of macro context"),
            Token::Data => abort!("Unexpected DATA directive"),
            Token::Equ => abort!("Unexpected EQU directive"),
            Token::Macro => abort!("Unexpected MACRO directive"),
            Token::EndMacro => abort!("Unexpected ENDMACRO directive"),
          }
        }
        Some(Directive::Data(data))
      }
      Some(Token::Equ) => {
        let name = match self.next_token() {
          Some(Token::Ident(s)) => to_string(s),
          Some(_) => abort!("Expected identifier for EQU directive"),
          None => abort!("Unexpected EOF"),
        };
        let constant = match self.next_token() {
          Some(tok) => match get_op_arg(tok) {
            Some(op) => op,
            None => abort!("Unexpected newline"),
          },
          None => abort!("Unexpected EOF"),
        };
        Some(Directive::Const(name, constant))
      }
      Some(Token::Rep) => abort!("Unexpected REP directive"),
      Some(Token::Comma) => abort!("Unexpected comma"),
      Some(Token::Here) => abort!("Unexpected $"),
      Some(Token::StrLit(_)) => abort!("Unexpected string literal"),
      Some(Token::NumLit(_)) => abort!("Unexpected number literal"),
      Some(Token::MacroArg(_)) => abort!("Unexpected macro argument"),
      Some(Token::Macro) | Some(Token::EndMacro) =>
        abort!("Macros are not yet implemented"),
      None => None,
    }
  }

  fn peek_char(&self) -> Option<u8> {
    self.input.get(self.idx).cloned()
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
          abort!("Unexpected `{}' ({})", ch as char, ch)
        } else {
          abort!("Unexpected EOF")
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
      Some(b'\n') => Some(Token::Newline),
      Some(b',') => Some(Token::Comma),
      Some(b'$') => Some(Token::Here),
      Some(quote) if quote == b'\'' || quote == b'"' => {
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
                    abort!("Unexpected EOF")
                  }
                }
              },
              Some(b'n') => buff.push(b'\n'),
              Some(ch) => abort!("Unrecogized escape sequence: \\{}", ch),
              None => abort!("Unexpected EOF"),
            }
          } else if ch == quote {
            break;
          } else {
            buff.push(ch);
          }
        }
        Some(Token::StrLit(buff))
      }
      Some(ch) if is_ident_start(ch) => {
        let mut ret = Vec::new();
        ret.push(to_upper(ch));
        while let Some(c) = self.peek_char() {
          if is_ident(c) {
            self.get_char();
            ret.push(to_upper(c));
          } else if c == b':' {
            self.get_char();
            return Some(Token::Label(ret));
          } else {
            break;
          }
        }
        if ret == b"DATA" {
          Some(Token::Data)
        } else if ret == b"EQU" {
          Some(Token::Equ)
        } else if ret == b"REP" {
          Some(Token::Rep)
        } else if ret == b"MACRO" {
          Some(Token::Macro)
        } else if ret == b"ENDMACRO" {
          Some(Token::EndMacro)
        } else {
          Some(Token::Ident(ret))
        }
      }
      Some(ch) if is_num(ch) => {
        let mut base = 10;
        let mut ret = Vec::new();
        if ch == b'0' {
          match self.peek_char().map(to_upper) {
            Some(b'B') => base = 2,
            Some(b'O') => base = 8,
            Some(b'D') => base = 10,
            Some(b'X') => base = 16,
            Some(ch) if is_num(ch) => ret.push(ch),
            Some(ch) if is_alpha(ch) => abort!(
              "Unsupported character in a base-{} literal: {}",
              base,
              ch as char,
            ),
            Some(_) | None => return Some(Token::NumLit(0)),
          }
          self.get_char();
        }

        while let Some(ch) = self.peek_char().map(to_upper) {
          if is_allowed(ch, base) {
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
        let ret = ret.iter().fold(0, |acc: u16, &el: &u8| {
          let add = if is_alpha(el) { el - b'A' + 10 } else { el - b'0' };
          match acc.checked_mul(base).and_then(|a| a.checked_add(add as u16)) {
            Some(a) => a,
            None => {
              abort!(
                "Attempted to write an overflowing number literal: {}",
                ::std::str::from_utf8(&ret).unwrap(),
              );
            }
          }
        });
        Some(Token::NumLit(ret))
      }
      Some(ch) => {
        abort!("Unsupported character: `{}' ({})", ch as char, ch);
      }
      None => None,
    }
  }
}
