use phf::phf_set;
use std::{collections::HashSet, env, io, process};

type PatternChars<'a> = std::iter::Peekable<core::str::Chars<'a>>;

static ALPHANUMERIC_CHARACTERS: phf::Set<char> = phf_set! {
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '_',
};

static NUMERIC_CHARACTERS: phf::Set<char> = phf_set! {
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
};

static VALID_CHARACTERS: phf::Set<char> = phf_set! {
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '_', '-', ' ',
};

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let parsed_patterns = parse_patterns(&pattern);

    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    let mut input_chars = input_line.chars().peekable();

    match parsed_patterns[0] {
        Pattern::BeginningOfLine => {
            if match_patterns(&mut input_chars.clone(), &parsed_patterns[1..]) {
                process::exit(0);
            }
        }
        _ => {
            while input_chars.peek().is_some() {
                if match_patterns(&mut input_chars.clone(), &parsed_patterns) {
                    process::exit(0);
                }

                input_chars.next();
            }
        }
    }

    process::exit(1);
}

#[derive(Clone, Debug)]
enum Pattern {
    Character(char),
    EscapeCharacter(char),
    CharacterGroup {
        characters: HashSet<char>,
        positive: bool,
    },
    BeginningOfLine,
    EndOfLine,
}

fn parse_patterns(pattern: &str) -> Vec<Pattern> {
    let mut result: Vec<Pattern> = vec![];
    let mut pattern_characters = pattern.chars().peekable();

    while let Some(pattern_char) = pattern_characters.next() {
        match pattern_char {
            c if VALID_CHARACTERS.contains(&c) => result.push(Pattern::Character(c)),
            '\\' => {
                if let Some(next_char) = pattern_characters.next() {
                    result.push(Pattern::EscapeCharacter(next_char))
                }
            }
            '[' => {
                let is_negative_group = *pattern_characters.peek().unwrap() == '^';

                let mut group_chars = HashSet::new();
                let mut found_closing_bracket = false;

                for group_char in pattern_characters.by_ref() {
                    match group_char {
                        ']' => {
                            found_closing_bracket = true;
                            break;
                        }
                        c => {
                            group_chars.insert(c);
                        }
                    }
                }

                if !found_closing_bracket {
                    panic!("Character group never closed");
                }

                result.push(Pattern::CharacterGroup {
                    characters: group_chars,
                    positive: !is_negative_group,
                });
            }
            '^' => result.push(Pattern::BeginningOfLine),
            '$' => result.push(Pattern::EndOfLine),
            _ => panic!("Unhandled symbol: {}", pattern_char),
        }
    }

    result
}

fn match_patterns(input_characters: &mut PatternChars, patterns: &[Pattern]) -> bool {
    patterns
        .iter()
        .all(|pattern| match_pattern(input_characters, pattern))
}

fn match_pattern(input_characters: &mut PatternChars, pattern: &Pattern) -> bool {
    match pattern {
        // Match our basic valid characters
        Pattern::Character(c) => {
            if let Some(next_char) = input_characters.next() {
                &next_char == c
            } else {
                false
            }
        }
        // Match escape patterns
        Pattern::EscapeCharacter(c) => match_escape_pattern(input_characters, c),
        // Match character groups
        Pattern::CharacterGroup {
            characters,
            positive,
        } => {
            if let Some(next_char) = input_characters.next() {
                match positive {
                    true => characters.contains(&next_char),
                    false => !characters.contains(&next_char),
                }
            } else {
                false
            }
        }
        Pattern::BeginningOfLine => false,
        Pattern::EndOfLine => {
            if let Some(next_char) = input_characters.next() {
                next_char == '\n'
            } else {
                true
            }
        }
    }
}

fn match_escape_pattern(input_characters: &mut PatternChars, escape_pattern: &char) -> bool {
    if let Some(next_char) = input_characters.next() {
        match escape_pattern {
            'd' => NUMERIC_CHARACTERS.contains(&next_char),
            'w' => ALPHANUMERIC_CHARACTERS.contains(&next_char),
            '\\' => next_char == '\\',
            _ => panic!("Unhandled escape pattern: {}", escape_pattern),
        }
    } else {
        false
    }
}
