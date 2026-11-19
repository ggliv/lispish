use crate::error::{LispishError, Result};

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    LParen,
    RParen,
    Quote,
    Symbol(String),
    Number(f64),
    String(String),
    Bool(bool),
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum State {
    Comment,
    String,
    /// Backslash in string
    StringEscape,
    Atom,
    Idle,
}

fn is_delim(c: char) -> bool {
    c.is_whitespace() || matches!(c, '(' | ')' | '\'' | ';' | '"')
}

fn transition(s: State, c: char) -> State {
    match s {
        State::Comment if c == '\n' => State::Idle,
        State::Comment => State::Comment,
        State::String if c == '"' => State::Idle,
        State::String if c == '\\' => State::StringEscape,
        State::String => State::String,
        State::StringEscape => State::String,
        State::Atom if c == ';' => State::Comment,
        State::Atom if c == '"' => State::String,
        State::Atom if is_delim(c) => State::Idle,
        State::Atom => State::Atom,
        State::Idle if c == ';' => State::Comment,
        State::Idle if c == '"' => State::String,
        State::Idle if is_delim(c) => State::Idle,
        State::Idle => State::Atom,
    }
}

/// try to get a symbol, number, or bool out of a lexed atom
fn try_parse_atom(atom: &str) -> Result<Token> {
    // must check is_finite so we don't match `nan`, `inf` and their rhymes
    if let Ok(n) = atom.parse::<f64>()
        && n.is_finite()
    {
        return Ok(Token::Number(n));
    }

    let mut chars = atom.chars();
    let is_numberish = match (chars.next(), chars.next()) {
        (Some('+') | Some('-'), Some(c)) => c.is_ascii_digit(),
        (Some(c), _) => c.is_ascii_digit(),
        _ => false,
    };

    if is_numberish {
        return Err(LispishError::InvalidNumber(atom.into()));
    }

    Ok(match atom {
        "#t" => Token::Bool(true),
        "#f" => Token::Bool(false),
        _ => Token::Symbol(atom.into()),
    })
}

pub fn lex(raw: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut state = State::Idle;

    // string for work-in-progress tokens that need one
    let mut wip_str = String::new();

    let handle_delim = |c: char, tokens: &mut Vec<_>| {
        match c {
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '\'' => tokens.push(Token::Quote),
            '"' | ';' => (),
            _ if c.is_whitespace() => (),
            // unreachable in the state machine
            _ => unreachable!("non-delim character `{:?}` reached handle_delim", c),
        }
    };

    for c in raw.chars() {
        let old_state = state;
        state = transition(state, c);

        match (old_state, state) {
            (State::String, State::String) => wip_str.push(c),
            (State::String, State::StringEscape) => (),
            (State::StringEscape, State::String) => match c {
                '"' | '\\' => wip_str.push(c),
                'n' => wip_str.push('\n'),
                'r' => wip_str.push('\r'),
                't' => wip_str.push('\t'),
                _ => return Err(LispishError::InvalidEscapeSequence(c)),
            },
            (State::String, _) => tokens.push(Token::String(std::mem::take(&mut wip_str))),
            (_, State::Atom) => wip_str.push(c),
            (State::Atom, _) => {
                tokens.push(try_parse_atom(std::mem::take(&mut wip_str).as_str())?);
                // ending an atom means we hit a delimiter
                handle_delim(c, &mut tokens);
            }
            (_, State::String) | (_, State::Comment) => (),
            (_, _) => handle_delim(c, &mut tokens),
        }
    }

    match state {
        State::String | State::StringEscape => {
            return Err(LispishError::UnterminatedString(
                wip_str.chars().take(10).chain("...".chars()).collect(),
            ));
        }
        State::Atom => tokens.push(try_parse_atom(&wip_str)?),
        _ => (),
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_whitespace_only() {
        assert_eq!(lex(""), Ok(vec![]));
        assert_eq!(lex("   \t\n\r  "), Ok(vec![]));
    }

    #[test]
    fn comments_only_and_inline_comments() {
        assert_eq!(lex("; just a comment"), Ok(vec![]));
        assert_eq!(lex("; comment with newline\n"), Ok(vec![]));
        assert_eq!(
            lex("(+ 1 2) ; trailing comment\n(* 3 4)"),
            Ok(vec![
                Token::LParen,
                Token::Symbol("+".into()),
                Token::Number(1.0),
                Token::Number(2.0),
                Token::RParen,
                Token::LParen,
                Token::Symbol("*".into()),
                Token::Number(3.0),
                Token::Number(4.0),
                Token::RParen,
            ])
        );
    }

    #[test]
    fn delimiters_touching_atoms_without_spaces() {
        assert_eq!(
            lex("'(foo(bar))"),
            Ok(vec![
                Token::Quote,
                Token::LParen,
                Token::Symbol("foo".into()),
                Token::LParen,
                Token::Symbol("bar".into()),
                Token::RParen,
                Token::RParen,
            ])
        );

        assert_eq!(
            lex("foo\"bar\"baz;comment\n123"),
            Ok(vec![
                Token::Symbol("foo".into()),
                Token::String("bar".into()),
                Token::Symbol("baz".into()),
                Token::Number(123.0),
            ])
        );
    }

    #[test]
    fn numbers_signed_floats_and_eof() {
        assert_eq!(
            lex("42 -7 +3.5 0.001"),
            Ok(vec![
                Token::Number(42.0),
                Token::Number(-7.0),
                Token::Number(3.5),
                Token::Number(0.001),
            ])
        );
        assert_eq!(lex("-99"), Ok(vec![Token::Number(-99.0)]));
    }

    #[test]
    fn symbols_and_inf_nan_words() {
        assert_eq!(
            lex("+ - -> ->> null? set! foo123 inf -inf nan NaN"),
            Ok(vec![
                Token::Symbol("+".into()),
                Token::Symbol("-".into()),
                Token::Symbol("->".into()),
                Token::Symbol("->>".into()),
                Token::Symbol("null?".into()),
                Token::Symbol("set!".into()),
                Token::Symbol("foo123".into()),
                Token::Symbol("inf".into()),
                Token::Symbol("-inf".into()),
                Token::Symbol("nan".into()),
                Token::Symbol("NaN".into()),
            ])
        );
    }

    #[test]
    fn booleans() {
        assert_eq!(
            lex("(#t #f)"),
            Ok(vec![
                Token::LParen,
                Token::Bool(true),
                Token::Bool(false),
                Token::RParen,
            ])
        );
    }

    #[test]
    fn strings_and_escapes() {
        assert_eq!(
            lex(r#""" "hello" "say \"hi\"" "C:\\path\\""#),
            Ok(vec![
                Token::String("".into()),
                Token::String("hello".into()),
                Token::String(r#"say "hi""#.into()),
                Token::String(r#"C:\path\"#.into()),
            ])
        );
    }

    #[test]
    fn invalid_numbers_return_error() {
        assert_eq!(lex("1hi"), Err(LispishError::InvalidNumber("1hi".into())));
        assert_eq!(
            lex("-12abc"),
            Err(LispishError::InvalidNumber("-12abc".into()))
        );
        assert_eq!(
            lex("+3.14.5"),
            Err(LispishError::InvalidNumber("+3.14.5".into()))
        );
    }

    #[test]
    fn unterminated_strings_return_error() {
        assert!(matches!(
            lex(r#""unclosed string"#),
            Err(LispishError::UnterminatedString(_))
        ));
        assert!(matches!(
            lex(r#""trailing escape\"#),
            Err(LispishError::UnterminatedString(_))
        ));
    }
}
