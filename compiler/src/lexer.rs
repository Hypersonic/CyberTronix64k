use std::fs::File;
use std::process::exit;
use std::io::Read;

use token::Token;

pub struct Lexer {
	pub buff: [u8; 32],
	pub start: u16,
	pub end: u16,
	pub file: File,
}

impl Lexer {
  /**
    new
    initializes a lexer, pointing to the start of file, and returns it
  */
  pub fn new(file: &str) -> Self {
    let file = match File::open(file) {
      Ok(f) => f,
      Err(e) => {
        eprintln!("fatal error - failed to open file `{}' ({})", file, e);
        exit(1);
      },
    };
    let mut lexer = Lexer {
      buff: [0; 32],
      start: 0,
      end: 0,
      file: file,
    };
    lexer.peekc();
    lexer
  }

  /**
    peekc
    Peek at the next character in the filestream without
    moving the line along.
  */
  fn peekc(&mut self) -> Option<u8> {
    debug!("entering peekc(L_start: {}, L_buff: {:p})", self.start, &self.buff);
    let mut ret = None;
    if self.start != self.end {
      ret = Some(self.buff[self.start as usize]);
    } else {
      self.start = 0;
      self.end = match self.file.read(&mut self.buff) {
        Ok(n) => n as u16,
        Err(e) => {
          eprintln!("Error reading file: {}", e);
          exit(1);
        },
      };
      if self.end != 0 {
        ret = Some(self.buff[0]);
      }
    }
    debug!("leaving peekc");
    ret
  }

  /**
    getc
    get the next character from the filestream, skipping over it
  */
  fn getc(&mut self) -> Option<u8> {
    debug!("entering getc(start: {}, buff: {:p})", self.start, &self.buff);
    let ret = if let Some(r) = self.peekc() {
      self.start += 1;
      Some(r)
    } else {
      None
    };
    debug!("leaving getc\n");
    ret
  }

  /**
    nxtok
    pull the next token from the file
  */
  pub fn next_token(&mut self) -> Token {
    #[inline]
    fn isspace(c: u8) -> bool {
      c == b' '
      || c == b'\t'
      || c == b'\n'
      || c == 0x0B
      || c == 0x0C
      || c == b'\r'
    }
    #[inline]
    fn isdigit(c: u8) -> bool {
      c >= b'0' && c <= b'9'
    }
    fn isalpha(c: u8) -> bool {
      (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z')
    }
    #[inline]
    fn isident_start(c: u8) -> bool {
      isalpha(c) || c == b'_'
    }
    #[inline]
    fn isident(c: u8) -> bool {
      isident_start(c) || isdigit(c)
    }
    debug!("entering nxtok");

    let current: u8 = if let Some(c) = self.getc() {
      c
    } else {
      return Token::Eof;
    };
    let ret = match current {
      c if isspace(c) => {
        while let Some(c) = self.peekc() {
          if !isspace(c) {
            break;
          }
          self.getc();
        }
        self.next_token()
      },
      c if isident(c) => {
        let mut s = Vec::new();
        s.push(c);
        while let Some(c) = self.peekc() {
          if isident(c) {
            self.getc();
            s.push(c);
          } else {
            break;
          }
        }
        Token::Ident(String::from_utf8(s).expect("Invalid utf-8 found"))
      },
      c if isdigit(c) => {
        fn is_allowed(c: u8, base: u16) -> bool {
          match base {
            2 => c >= b'0' && c < b'2',
            8 => c >= b'0' && c < b'8',
            10 => isdigit(c),
            16 => isdigit(c)
              || (c >= b'A' && c <= b'F')
              || (c >= b'a' && c <= b'f'),
            _ => unreachable!(),
          }
        }
        let mut base = 10;
        let mut ret = Vec::new();
        if c == b'0' {
          match self.peekc() {
            Some(b'b') => base = 2,
            Some(b'o') => base = 8,
            Some(b'd') => base = 10,
            Some(b'x') => base = 16,
            Some(ch) if isdigit(ch) => ret.push(ch),
            Some(ch) if isident(ch) => {
              eprintln!("Unknown base specifier: {}", ch as char)
            },
            Some(_) | None => return Token::IntLit(0),
          }
          self.getc();
        } else {
          ret.push(c);
        }

        while let Some(ch) = self.peekc() {
          if is_allowed(ch, base) {
            self.getc();
            ret.push(ch);
          } else if isalpha(ch) {
            eprintln!(
              "Unsupported character in a base-{} literal: {}",
              base,
              ch as char,
            );
          } else {
            break;
          }
        }

        let ret = ret.iter().fold(0u16, |acc, &el| {
          let add = if el >= b'A' && el <= b'Z' {
            el - b'A' + 10
          } else if el >= b'a' && el <= b'z' {
            el - b'a' + 10
          } else {
            el - b'0'
          };
          match acc.checked_mul(base).and_then(|a| a.checked_add(add as u16)) {
            Some(a) => a,
            None => {
              eprintln!(
                "Attempted to write an overflowing number literal: {}",
                ::std::str::from_utf8(&ret).unwrap()
              );
              exit(1);
            }
          }
        });
        Token::IntLit(ret)
      },
      b'\'' => {
        let ret = match self.getc() {
          Some(b'\'') => {
            eprintln!("unescaped `'' in a char literal");
            exit(1);
          },
          Some(b'\\') => match self.getc() {
            Some(b'n') => b'\n',
            Some(b'r') => b'\r',
            Some(b't') => b'\t',
            Some(b'\\') => b'\\',
            Some(b'0') => b'\0',
            Some(c) => {
              eprintln!("Unsupported escape code: \\{}", c as char);
              exit(1);
            },
            None => {
              eprintln!("Unexpected EOF");
              exit(1);
            }
          },
          Some(c) => c,
          None => {
            eprintln!("Unexpected EOF");
            exit(1);
          },
        };
        if self.getc() != Some(b'\'') {
          eprintln!("Expected end of char literal");
          exit(1);
        }
        Token::CharLit(ret)
      }
      b'\"' => {
        let mut ret = Vec::new();
        loop {
          match self.getc() {
            Some(b'\\') => match self.getc() {
              Some(b'n') => ret.push(b'\n'),
              Some(b'r') => ret.push(b'\r'),
              Some(b't') => ret.push(b'\t'),
              Some(b'\\') => ret.push(b'\\'),
              Some(b'0') => ret.push(b'\0'),
              Some(b'\n') => {},
              Some(c) => {
                eprintln!("Unsupported escape code: \\{}", c as char);
                exit(1);
              },
              None => {
                eprintln!("Unexpected EOF");
                exit(1);
              }
            },
            Some(b'"') => break,
            Some(c) => ret.push(c),
            None => {
              eprintln!("Unexpected EOF");
              exit(1);
            },
          };
          if self.getc() != Some(b'\'') {
            eprintln!("Expected end of char literal");
            exit(1);
          }
        }
        Token::StringLit(String::from_utf8(ret).expect("Invalid utf-8 found"))
      }
      b'=' => if let Some(b'=') = self.peekc() {
        eprintln!("== unimplemented");
        exit(1);
      } else {
        Token::Equal
      },
      b'+' => if let Some(b'=') = self.peekc() {
        self.getc();
        Token::PlusEqual
      } else {
        Token::Plus
      },
      b'-' => if let Some(b'=') = self.peekc() {
        self.getc();
        Token::MinusEqual
      } else {
        Token::Minus
      },
      b'*' => if let Some(b'=') = self.peekc() {
        self.getc();
        Token::MultiplyEqual
      } else {
        Token::Multiply
      },
      b'/' => if let Some(b'=') = self.peekc() {
        self.getc();
        Token::DivideEqual
      } else if let Some(b'/') = self.peekc() {
        self.comment()
      } else {
        Token::Divide
      },
      b'%' => if let Some(b'=') = self.peekc() {
        self.getc();
        Token::RemainderEqual
      } else {
        Token::Remainder
      },
      b'&' => if let Some(b'&') = self.peekc() {
        eprintln!("&& unimplemented");
        exit(1);
      } else {
        Token::Reference
      },
      b'(' => Token::OpenParen,
      b')' => Token::CloseParen,
      b'{' => Token::OpenBrace,
      b'}' => Token::CloseBrace,
      b':' => Token::Colon,
      b';' => Token::Semicolon,
      c => Token::Unknown(c),
    };

    debug!("leaving nxtok\n");

    ret
  }

  /**
    comment
    recursively finds the end of a comment, and gets the next token.
    i.e., /* /* */ */ is a valid comment.
  */
  fn comment(&mut self) -> Token {
    eprintln!("comments unimplemented");
    exit(1);
  }
}
