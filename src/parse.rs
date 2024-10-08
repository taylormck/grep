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
    let mut lhs = parse_base_token(token, tokens);

    while let Some(token) = tokens.peek() {
        match token {
            Token::Star => {
                lhs = Expression::Repeat(Box::from(lhs));
            }
            Token::Plus => {
                lhs = Expression::Sequence(vec![lhs.clone(), Expression::Repeat(Box::from(lhs))]);
            }
            Token::Question => {
                lhs = Expression::Optional(Box::from(lhs));
            }
            _ => break,
        };

        tokens.next();
    }

    lhs
}

fn parse_base_token(token: &Token, tokens: &mut TokenIter) -> Expression {
    match token {
        Token::Literal(c) => Expression::Literal(*c),
        Token::Space => Expression::Literal(' '),
        Token::Hyphen => Expression::Literal('-'),
        Token::Caret => Expression::BeginningOfLine,
        Token::Dollar => Expression::EndOfLine,
        Token::Dot => Expression::Wildcard,
        Token::BackSlash => {
            if let Some(token) = tokens.next() {
                return match token {
                    Token::Literal(d) if character_sets::NUMERIC_CHARACTERS.contains(d) => {
                        let mut num = format!("{}", d);

                        while let Some(token) = tokens.peek() {
                            match token {
                                Token::Literal(d)
                                    if character_sets::NUMERIC_CHARACTERS.contains(d) =>
                                {
                                    num.push(*d);
                                    tokens.next();
                                }
                                _ => break,
                            }
                        }
                        let num = match num.parse() {
                            Ok(n) => n,
                            Err(_) => panic!("Invalid number: {}", num),
                        };

                        Expression::BackReference(num)
                    }
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
            let mut current_expression: Vec<Expression> = vec![];

            loop {
                match tokens.next() {
                    Some(Token::CloseParen) => {
                        let expr = Expression::Sequence(current_expression);
                        inner_expressions.push(expr);

                        break;
                    }
                    Some(Token::Pipe) => {
                        let expr = Expression::Sequence(current_expression);
                        inner_expressions.push(expr);
                        current_expression = vec![];
                    }

                    Some(token) => {
                        let expr = parse_unary(token, tokens);
                        current_expression.push(expr);
                    }
                    None => panic!("Unended capture group!"),
                }
            }

            let expr = Expression::Alternation(inner_expressions);

            Expression::Capture(Box::from(expr))
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
        token => panic!("Invalid base token: {:?}", token),
    }
}
