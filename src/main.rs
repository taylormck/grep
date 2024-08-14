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

fn match_escape_pattern(input_line: &str, escape_pattern: &char) -> bool {
    let char = input_line.chars().next().unwrap();

    match escape_pattern {
        'd' => format!("{}", char).parse::<u32>().is_ok(),
        'w' => ALPHANUMERIC_CHARACTERS.contains(&char),
        _ => panic!("Unhandled escape pattern: {}", escape_pattern),
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    match pattern.len() {
        1 => input_line.contains(pattern),
        _ => {
            let mut chars = pattern.chars();
            let symbol = chars.next().unwrap();

            match symbol {
                '\\' => {
                    let sub_pattern = chars.next().unwrap();
                    match_escape_pattern(input_line, &sub_pattern)
                }
                '[' => {
                    if let Some(end_index) = pattern.find(']') {
                        let sub_pattern = &pattern[1..end_index];

                        match sub_pattern.starts_with('^') {
                            true => !sub_pattern[1..].chars().any(|character| {
                                match_pattern(input_line, &format!("{}", character))
                            }),

                            false => sub_pattern.chars().any(|character| {
                                match_pattern(input_line, &format!("{}", character))
                            }),
                        }
                    } else {
                        panic!("Found opening '[' but no closing ']'")
                    }
                }
                _ => panic!("Unhandled symbol: {}", symbol),
            }
        }
    }
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

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
