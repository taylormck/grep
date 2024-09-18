use crate::{character_sets, expression::Expression};
type CharIter<'a> = std::iter::Peekable<std::str::Chars<'a>>;

pub fn evaluate(expression: &Expression, input: &str) -> Option<String> {
    let mut chars = input.chars().peekable();
    let mut backreferences: Vec<String> = Vec::new();

    while chars.peek().is_some() {
        match evaluate_from_beginning(expression, &mut chars.clone(), &mut backreferences) {
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

fn evaluate_from_beginning(
    expression: &Expression,
    chars: &mut CharIter,
    backreferences: &mut Vec<String>,
) -> Option<String> {
    match expression {
        Expression::Empty => Some("".to_string()),
        Expression::Sequence(expressions) => {
            let mut results: Vec<String> = vec![];

            let mut remaining_expressions = expressions.iter();

            while let Some(expr) = remaining_expressions.next() {
                match expr {
                    Expression::Repeat(repeated_expr) => {
                        let mut stack = vec![("".to_string(), chars.clone())];

                        while let Some(result) =
                            evaluate_from_beginning(repeated_expr, chars, backreferences)
                        {
                            stack.push((result, chars.clone()));
                        }

                        let remaining_expressions: Vec<Expression> =
                            remaining_expressions.clone().cloned().collect();

                        let remaining_expressions = match remaining_expressions.len() {
                            0 => Expression::Empty,
                            _ => Expression::Sequence(remaining_expressions),
                        };

                        // Work our way back down the stack until we match the rest of the input
                        while let Some((matched_str, mut remaining_input)) = stack.pop() {
                            if let Some(result) = evaluate_from_beginning(
                                &remaining_expressions,
                                &mut remaining_input,
                                &mut backreferences.clone(),
                            ) {
                                results.push(matched_str);
                                results.push(result);

                                return Some(results.iter().map(String::from).collect());
                            }
                        }

                        return None;
                    }

                    _ => match evaluate_from_beginning(expr, chars, backreferences) {
                        Some(result) => {
                            results.push(result);
                        }
                        None => {
                            return None;
                        }
                    },
                }
            }

            Some(results.iter().map(String::from).collect())
        }
        Expression::Literal(c) => match chars.next() {
            Some(next_char) if next_char == *c => Some(String::from(*c)),
            Some('\n') => evaluate_from_beginning(expression, chars, backreferences),
            _ => None,
        },
        Expression::Wildcard => match chars.next() {
            Some(next_char) if character_sets::VALID_CHARACTERS.contains(&next_char) => {
                Some(String::from(next_char))
            }
            _ => None,
        },
        Expression::Escape(c) => match chars.next() {
            Some(next_char) if match_escape_pattern(next_char, c) => Some(String::from(next_char)),
            _ => None,
        },
        Expression::Capture(expr) => match evaluate_from_beginning(expr, chars, backreferences) {
            Some(result) => {
                backreferences.push(result.clone());
                Some(result)
            }
            _ => None,
        },

        Expression::CharacterGroup(group) => match chars.next() {
            Some(next_char) if group.contains(&next_char) => Some(String::from(next_char)),
            _ => None,
        },
        Expression::NegativeCharacterGroup(group) => match chars.next() {
            Some(next_char) if !group.contains(&next_char) => Some(String::from(next_char)),
            _ => None,
        },
        Expression::BeginningOfLine => match chars.next() {
            Some('\n') => Some("".to_string()),
            _ => None,
        },
        Expression::EndOfLine => match chars.next() {
            Some('\n') => Some("".to_string()),
            _ => None,
        },
        Expression::Repeat(_) => None,
        Expression::Optional(expr) => {
            let mut forward_chars = chars.clone();

            match evaluate_from_beginning(expr, &mut forward_chars, backreferences) {
                Some(result) => {
                    *chars = forward_chars;
                    Some(result)
                }
                None => Some("".to_string()),
            }
        }
        Expression::Alternation(exprs) => {
            for expr in exprs {
                let mut forward_chars = chars.clone();
                if let Some(result) =
                    evaluate_from_beginning(expr, &mut forward_chars, backreferences)
                {
                    *chars = forward_chars;
                    return Some(result);
                }
            }

            None
        }
        Expression::BackReference(num) => match backreferences.get(num - 1) {
            Some(backreference) => {
                let exprs = backreference.chars().map(Expression::Literal).collect();
                let expr = Expression::Sequence(exprs);

                evaluate_from_beginning(&expr, chars, backreferences)
            }
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
