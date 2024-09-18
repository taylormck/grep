use crate::{character_sets, token::Token};
use core::iter;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Sequence(Vec<Expression>),
    Literal(char),
    Escape(char),
    Capture(Box<Expression>),
}

pub fn parse(tokens: Vec<Token>) -> Expression {
    let mut expressions = vec![];
    let mut tokens = tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        let expr = parse_token(token, &mut tokens);
        expressions.push(expr);
    }

    Expression::Sequence(expressions)
}

type TokenIter<'a> = iter::Peekable<std::slice::Iter<'a, Token>>;

fn parse_token(token: &Token, tokens: &mut TokenIter) -> Expression {
    match token {
        Token::Literal(c) => Expression::Literal(*c),
        Token::BackSlash => {
            if let Some(token) = tokens.next() {
                return match token {
                    Token::Literal(c) if character_sets::VALID_ESCAPE_CHARACTERS.contains(c) => {
                        Expression::Escape(*c)
                    }
                    _ => panic!("Unsupported esacpe sequence."),
                };
            }

            panic!("Can't use a backslash at the end of the pattern.")
        }
        Token::OpenParen => {
            let mut inner_tokens = vec![];

            while let Some(token) = tokens.peek() {
                if token == &&Token::CloseParen {
                    tokens.next();
                    break;
                }
                inner_tokens.push(*tokens.next().unwrap());
            }

            Expression::Capture(Box::from(parse(inner_tokens)))
        }
        _ => todo!(),
    }
}
