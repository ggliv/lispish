mod error;
mod lexer;
mod parser;

use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let input = io::read_to_string(io::stdin())?;
    let tokens = lexer::lex(&input)?;
    let ast = parser::parse(tokens)?;
    println!("{ast:#?}");

    Ok(())
}
