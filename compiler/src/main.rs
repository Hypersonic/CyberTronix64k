/*
 * main
 * driver file for the rc compiler.
 */

#[macro_use]
mod macros;

mod lexer;
mod token;

use lexer::Lexer;
use token::Token;

fn main() {
  let args = std::env::args().collect::<Vec<_>>();
  let input: &str;
  if args.len() < 2 {
    eprintln!("fatal error - no input file");
    std::process::exit(1);
  } else if args.len() > 2 {
    eprintln!("fatal error - {} only supports one input file\n", args[0]);
    std::process::exit(1);
  } else {
    input = &args[1];
  }

  let mut lexer: Lexer = Lexer::new(input);
  let mut cur_tok: Token = lexer.next_token();
  loop { if let Token::Eof = cur_tok { break; }
    println!("{:?}", cur_tok);
    cur_tok = lexer.next_token();
  }
}

