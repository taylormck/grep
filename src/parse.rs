use crate::{character_sets, expression::Expression, token::Token};
use core::iter;
use std::collections::HashSet;

type TokenIter<'a> = iter::Peekable<std::slice::Iter<'a, Token>>;

pub fn parse(tokens: &[Token]) -> Expression {
    let mut expressions = vec![];
    let mut tokens = tokens.iter().peekable();

    while let Some(token) = tokens.next() {
        let expr = parse_unary(token, &mut tokens);
        expressions.push(expr);
    }

    Expression::Sequence(expressions)
}

fn parse_unary(token: &Token, tokens: &mut TokenIter) -> Expression {
    let mut lhs = parse_token(token, tokens);

    while let Some(token) = tokens.peek() {
        match token {
            Token::Star => {
                lhs = Expression::Repeat(Box::from(lhs));
                tokens.next();
            }
            Token::Plus => {
                lhs = Expression::Sequence(vec![lhs.clone(), Expression::Repeat(Box::from(lhs))]);
                tokens.next();
            }
            _ => break,
        };
    }

    lhs
}

fn parse_token(token: &Token, tokens: &mut TokenIter) -> Expression {
    match token {
        Token::Literal(c) => Expression::Literal(*c),
        Token::Caret => Expression::BeginningOfLine,
        Token::Dollar => Expression::EndOfLine,
        Token::BackSlash => {
            if let Some(token) = tokens.next() {
                return match token {
                    Token::Literal(c) if character_sets::VALID_ESCAPE_CHARACTERS.contains(c) => {
                        Expression::Escape(*c)
                    }
                    Token::BackSlash => Expression::Escape('\\'),
                    _ => panic!("Unsupported esacpe sequence."),
                };
            }

            panic!("Can't use a backslash at the end of the pattern.")
        }
        Token::OpenParen => {
            let mut inner_expressions: Vec<Expression> = vec![];

            loop {
                match tokens.next() {
                    Some(Token::CloseParen) => break,
                    Some(token) => {
                        let expr = parse_token(token, tokens);
                        inner_expressions.push(expr);
                    }
                    None => panic!("Unended capture group!"),
                }
            }

            Expression::Capture(Box::from(Expression::Sequence(inner_expressions)))
        }
        Token::OpenBracket => {
            let mut group_chars: HashSet<char> = HashSet::new();

            let is_negative = match tokens.peek() {
                Some(Token::Caret) => {
                    tokens.next();
                    true
                }
                _ => false,
            };

            loop {
                match tokens.next() {
                    Some(Token::CloseBracket) => {
                        break;
                    }
                    Some(Token::Literal(c)) => {
                        group_chars.insert(*c);
                    }
                    None => panic!("Unended character group!"),
                    _ => panic!("Unsupported token in character group!"),
                }
            }

            match is_negative {
                true => Expression::NegativeCharacterGroup(group_chars),
                false => Expression::CharacterGroup(group_chars),
            }
        }
        _ => todo!(),
    }
}
