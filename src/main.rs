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

fn match_pattern(input_characters: &mut PatternChars, pattern: &str) -> bool {
    let mut pattern_characters = pattern.chars().peekable();

    while let Some(pattern_char) = pattern_characters.next() {
        if !match pattern_char {
            // Match our basic valid characters
            c if VALID_CHARACTERS.contains(&c) => {
                if let Some(next_char) = input_characters.next() {
                    next_char == pattern_char
                } else {
                    false
                }
            }
            // Match escape patterns
            '\\' => {
                let sub_pattern = pattern_characters.next().unwrap();
                match_escape_pattern(input_characters, &sub_pattern)
            }
            // Match character groups
            '[' => {
                let negative_group = *pattern_characters.peek().unwrap() == '^';

                let mut group_chars = HashSet::new();
                for group_char in pattern_characters.by_ref() {
                    match group_char {
                        ']' => break,
                        c => {
                            group_chars.insert(c);
                        }
                    }
                }

                if let Some(next_char) = input_characters.next() {
                    match negative_group {
                        true => !group_chars.contains(&next_char),
                        false => group_chars.contains(&next_char),
                    }
                } else {
                    false
                }
            }
            _ => panic!("Unhandled symbol: {}", pattern_char),
        } {
            return false;
        }
    }

    true
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    let mut input_chars = input_line.chars().peekable();

    match pattern.chars().next() {
        Some('^') => {
            if match_pattern(&mut input_chars.clone(), &pattern[1..]) {
                process::exit(0);
            }
        }
        Some(_) => {
            while input_chars.peek().is_some() {
                if match_pattern(&mut input_chars.clone(), &pattern) {
                    process::exit(0);
                }

                input_chars.next();
            }
        }
        _ => {
            process::exit(1);
        }
    }

    process::exit(1);
}
