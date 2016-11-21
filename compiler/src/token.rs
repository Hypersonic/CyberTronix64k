use std::fmt::{self, Debug};

pub enum Token {
	Ident(String), /* identifier */

	/* literals */
	IntLit(u16), /* integer literal */
	CharLit(u8), /* character literal */
	StringLit(String), /* string literal */

	/* operators */
	Plus, /* plus operator `+' */
	Minus, /* minus operator `-' */
	Multiply, /* multiplication operator `*' */
	Divide, /* division operator `/' */
	Remainder, /* remainder operator `%' */
	Reference, /* ref operator `&' */

	/* equal operators */
	PlusEqual, /* plus-equal operator `+=' */
	MinusEqual, /* minus-equal operator `-=' */
	MultiplyEqual, /* mul-equal operator `*=' */
	DivideEqual, /* div-equal operator `/=' */
	RemainderEqual, /* rem-equal operator `%=' */
	Equal, /* equal operator `=' */

	/* brackets */
	OpenParen, /* open paren `(' */
	CloseParen, /* close paren `)' */
	OpenBrace, /* open brace `{' */
	CloseBrace, /* close brace `}' */

	/* miscellaneous */
	Colon, /* colon */
	Semicolon, /* semicolon */

	Eof, /* end of file */

	Unknown(u8), /* unknown character */
}

impl Debug for Token {
  /**
    T_fprt
    prints a token to stream
    formatted like <token-type[: more-info]>
   */
  fn fmt(&self, stream: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Token::Ident(ref s) => write!(stream, "<ident: {}>", s),
      Token::IntLit(i) => write!(stream, "<int literal: {}>", i),
      Token::CharLit(c) => {
        try!(write!(stream, "<char literal: `"));
        try!(escaped_char(stream, c));
        write!(stream, "`>")
      },
      Token::StringLit(ref s) => write!(stream, "<strlit: `{}'>", s),
      Token::Plus => write!(stream, "<plus `+'>"),
      Token::Minus => write!(stream, "<minus `-'>"),
      Token::Multiply => write!(stream, "<multiply `*'>"),
      Token::Divide => write!(stream, "<divide `/'>"),
      Token::Remainder => write!(stream, "<remainder `%'>"),
      Token::Reference => write!(stream, "<reference `&'>"),
      Token::PlusEqual => write!(stream, "<plus-equal `+='>"),
      Token::MinusEqual => write!(stream, "<minus-equal `-='>"),
      Token::MultiplyEqual => write!(stream, "multiply-equal `*='>"),
      Token::DivideEqual => write!(stream, "divide-equal `/='>"),
      Token::RemainderEqual => write!(stream, "remainder-equal `%%='>"),
      Token::Equal => write!(stream, "<equal `='>"),
      Token::OpenParen => write!(stream, "<open paren>"),
      Token::CloseParen => write!(stream, "<close paren>"),
      Token::OpenBrace => write!(stream, "<open brace>"),
      Token::CloseBrace => write!(stream, "<close brace>"),
      Token::Colon => write!(stream, "<colon>"),
      Token::Semicolon => write!(stream, "<semicolon>"),
      Token::Eof => write!(stream, "<eof>"),
      Token::Unknown(c) => {
        try!(write!(stream, "<unknown character: `"));
        try!(escaped_char(stream, c));
        write!(stream, "`>")
      }
    }
  }
}

/**
  escaped_char
  print a char's escape sequence, instead of the char itself.
 */
fn escaped_char(stream: &mut fmt::Formatter, c: u8) -> fmt::Result {
  match c {
    b'\0' => stream.write_str("\\0"),
    b'\t' => stream.write_str("\\t"),
    b'\n' => stream.write_str("\\n"),
    b'\r' => stream.write_str("\\r"),
    1...9 | 0xE...0x19 | 0x7E...0xFF => write!(stream, "\\x{:02x}", c),
    12 => stream.write_str("\\f"),
    _ => stream.write_fmt(format_args!("{}", c as char)),
  }
}
