use crate::{character_sets, token::Token};
use core::iter;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Sequence(Vec<Expression>),
    Literal(char),
    Escape(char),
    Capture(Box<Expression>),
}

pub fn parse(tokens: &[Token]) -> Expression {
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

            Expression::Capture(Box::from(parse(&inner_tokens)))
        }
        _ => todo!(),
    }
}

type CharIter<'a> = std::iter::Peekable<std::str::Chars<'a>>;

pub fn evaluate(expression: &Expression, input: &str) -> Option<String> {
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        match evaluate_from_beginning(expression, &mut chars) {
            Some(result) => {
                return Some(result);
            }
            None => {
                chars.next();
            }
        }
    }
    None
}

fn evaluate_from_beginning(expression: &Expression, chars: &mut CharIter) -> Option<String> {
    match expression {
        Expression::Sequence(expressions) => {
            let mut results: Vec<String> = vec![];

            for expr in expressions {
                match evaluate_from_beginning(expr, chars) {
                    Some(result) => {
                        results.push(result);
                    }
                    None => {
                        return None;
                    }
                }
            }

            Some(results.iter().map(String::from).collect())
        }
        Expression::Literal(c) => match chars.next() {
            Some(next_char) if next_char == *c => Some(String::from(*c)),
            Some(next_char) => {
                dbg!(c, next_char);
                None
            }
            _ => None,
        },
        // Expression::Escape(c) => match chars.next() {
        //     Some(next_char) if next_char == *c => Some(String::from(*c)),
        //     _ => None,
        // },
        Expression::Capture(expr) => evaluate_from_beginning(expr, chars),
        _ => todo!(),
    }
}

// fn match_escape_pattern(next_char: char, escape_pattern: &char) -> bool {
//     match escape_pattern {
//         'd' => NUMERIC_CHARACTERS.contains(&next_char),
//         'w' => ALPHANUMERIC_CHARACTERS.contains(&next_char),
//         '\\' => next_char == '\\',
//         _ => panic!("Unhandled escape pattern: {}", escape_pattern),
//     }
// }
//
// fn match_character_group(next_char: char, group: &CharacterGroup) -> bool {
//     match group.positive {
//         true => group.characters.contains(&next_char),
//         false => !group.characters.contains(&next_char),
//     }
// }
