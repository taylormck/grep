use crate::{character_sets, token::Token};
use core::iter;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Empty,
    Sequence(Vec<Expression>),
    Literal(char),
    Escape(char),
    Capture(Box<Expression>),
    CharacterGroup(HashSet<char>),
    NegativeCharacterGroup(HashSet<char>),
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
                    Token::BackSlash => Expression::Escape('\\'),
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
        Expression::Empty => Some("".to_string()),
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
        Expression::Escape(c) => match chars.next() {
            Some(next_char) if match_escape_pattern(next_char, c) => Some(String::from(next_char)),
            _ => None,
        },
        Expression::Capture(expr) => evaluate_from_beginning(expr, chars),
        Expression::CharacterGroup(group) => match chars.next() {
            Some(next_char) if group.contains(&next_char) => Some(String::from(next_char)),
            _ => None,
        },
        Expression::NegativeCharacterGroup(group) => match chars.next() {
            Some(next_char) if !group.contains(&next_char) => Some(String::from(next_char)),
            _ => None,
        },
    }
}

fn match_escape_pattern(next_char: char, escape_pattern: &char) -> bool {
    match escape_pattern {
        'd' => character_sets::NUMERIC_CHARACTERS.contains(&next_char),
        'w' => character_sets::ALPHANUMERIC_CHARACTERS.contains(&next_char),
        '\\' => next_char == '\\',
        _ => panic!("Unhandled escape pattern: {}", escape_pattern),
    }
}

// fn match_character_group(next_char: char, group: &CharacterGroup) -> bool {
//     match group.positive {
//         true => group.characters.contains(&next_char),
//         false => !group.characters.contains(&next_char),
//     }
// }
