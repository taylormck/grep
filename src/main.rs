use grep_starter_rust::{evaluate, parse, token};
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
    let expression = parse(&pattern_tokens);

    // dbg!(pattern_tokens, &expression);

    let input = format!("\n{}\n", input);

    match evaluate(&expression, &input) {
        Some(result) => {
            println!("Match found: {}", result);
        }
        None => {
            println!("No match!");
            process::exit(1);
        }
    }

    process::exit(1);
}
