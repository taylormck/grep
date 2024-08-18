use phf::phf_set;
use std::env;
use std::io;
use std::process;

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
    '5', '6', '7', '8', '9', '_', '-', ' '
};

fn match_escape_pattern(input_characters: &mut core::str::Chars, escape_pattern: &char) -> bool {
    match escape_pattern {
        'd' => input_characters.any(|char| NUMERIC_CHARACTERS.contains(&char)),
        'w' => input_characters.any(|char| ALPHANUMERIC_CHARACTERS.contains(&char)),
        _ => panic!("Unhandled escape pattern: {}", escape_pattern),
    }
}

fn match_pattern(input_characters: &mut core::str::Chars, pattern: &str) -> bool {
    let mut pattern_characters = pattern.chars();

    while let Some(pattern_char) = pattern_characters.next() {
        if !match pattern_char {
            // Match our basic valid characters
            c if VALID_CHARACTERS.contains(&c) => input_characters.next().unwrap() == pattern_char,
            // Match escape patterns
            '\\' => {
                let sub_pattern = pattern_characters.next().unwrap();
                match_escape_pattern(input_characters, &sub_pattern)
            }
            // Match character groups
            '[' => {
                // BUG: this will stop at the first ']' character, so it'll
                // break if we have more than one character group.
                if let Some(end_index) = pattern.find(']') {
                    let sub_pattern = &pattern[1..end_index];

                    match sub_pattern.starts_with('^') {
                        true => !sub_pattern[1..].chars().any(|character| {
                            match_pattern(input_characters, &format!("{}", character))
                        }),

                        false => sub_pattern.chars().any(|character| {
                            match_pattern(input_characters, &format!("{}", character))
                        }),
                    }
                } else {
                    panic!("Unmatched '['")
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

    let mut input_chars = input_line.chars();

    while input_chars.next().is_some() {
        if match_pattern(&mut input_chars, &pattern) {
            process::exit(0)
        }
    }

    process::exit(1)
}
