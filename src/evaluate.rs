use crate::{character_sets, expression::Expression};
type CharIter<'a> = std::iter::Peekable<std::str::Chars<'a>>;

pub fn evaluate(expression: &Expression, input: &str) -> Option<String> {
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        match evaluate_from_beginning(expression, &mut chars.clone()) {
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
