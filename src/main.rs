use grep_starter_rust::{expression, token};
use std::{env, io, process};

// type PatternChas<'a> = std::iter::Peekable<str::Chars<'a>>;
// type PatternsIter<'a> = core::slice::Iter<'a, Pattern>;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        eprintln!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = match env::args().nth(2) {
        Some(pattern) => pattern,
        None => {
            eprintln!("Expected second argument to be a pattern");
            process::exit(1);
        }
    };

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let pattern_tokens = token::tokenize(&pattern);
    let expression = expression::parse(pattern_tokens);

    dbg!(expression);

    process::exit(1);
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
