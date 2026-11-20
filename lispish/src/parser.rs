use crate::error::{LispishError, Result};
use crate::lexer::Token;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Symbol(String),
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<Expr>),
}

pub fn parse_expr<'a, T: Iterator<Item = &'a Token>>(
    tokens: &mut std::iter::Peekable<T>,
) -> Result<Expr> {
    let token = tokens.next().ok_or(LispishError::UnexpectedEof)?;

    match token {
        Token::LParen => {
            let mut list = Vec::new();
            while let Some(t) = tokens.peek()
                && **t != Token::RParen
            {
                list.push(parse_expr(tokens)?);
            }
            if tokens.peek().is_none() {
                Err(LispishError::UnexpectedEof)
            } else {
                // consume rparen
                tokens.next();
                Ok(Expr::List(list))
            }
        }
        // lparent match above should consume all rparens
        Token::RParen => Err(LispishError::UnexpectedToken(token.clone())),
        Token::Quote => {
            let next_expr = parse_expr(tokens)?;
            Ok(Expr::List(vec![Expr::Symbol("quote".into()), next_expr]))
        }
        Token::Symbol(s) => Ok(Expr::Symbol(s.clone())),
        Token::Number(n) => Ok(Expr::Number(*n)),
        Token::String(s) => Ok(Expr::String(s.clone())),
        Token::Bool(b) => Ok(Expr::Bool(*b)),
    }
}

pub fn parse<'a>(tokens: impl IntoIterator<Item = &'a Token>) -> Result<Vec<Expr>> {
    let mut exprs = Vec::new();

    let mut iter = tokens.into_iter().peekable();

    while iter.peek().is_some() {
        exprs.push(parse_expr(&mut iter)?)
    }

    Ok(exprs)
}
