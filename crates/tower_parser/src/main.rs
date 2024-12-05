use std::{
  error::Error,
  fs::{read_to_string, File},
  io::Write,
};

use bumpalo::Bump;
use tower_parser::parser::{ast::SourceType, parse_source};

fn main() -> Result<(), Box<dyn Error>> {
  let allocator = Bump::new();
  let source_string = read_to_string("./example/hello.js")?;
  let source_chars = source_string.chars().collect::<Vec<char>>();

  let program = parse_source(&allocator, &source_chars, SourceType::Module)?;

  File::create("./example/hello.ast")?.write_fmt(format_args!("{program:#?}"))?;

  Ok(())
}
