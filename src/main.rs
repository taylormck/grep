use phf::phf_set;
use std::{collections::HashSet, env, io, process};

type PatternChars<'a> = std::iter::Peekable<core::str::Chars<'a>>;
type PatternsIter<'a> = std::slice::Iter<'a, Pattern>;

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

static VALID_ESCAPE_CHARACTERS: phf::Set<char> = phf_set! {
    'd', 'w', '\\',
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
        Pattern::BasicPattern(BasicPattern::BeginningOfLine) => {
            if match_patterns(&mut input_chars.clone(), &mut parsed_patterns[1..].iter()) {
                process::exit(0);
            }
        }
        _ => {
            while input_chars.peek().is_some() {
                if match_patterns(&mut input_chars.clone(), &mut parsed_patterns.iter()) {
                    process::exit(0);
                }

                input_chars.next();
            }
        }
    }

    process::exit(1);
}

#[derive(Clone, Debug)]
enum BasicPattern {
    Character(char),
    EscapeCharacter(char),
    CharacterGroup(CharacterGroup),
    BeginningOfLine,
    EndOfLine,
    Wildcard,
}

#[derive(Clone, Debug)]
enum Pattern {
    BasicPattern(BasicPattern),
    ZeroOrMore(BasicPattern),
}

#[derive(Clone, Debug)]
struct CharacterGroup {
    characters: HashSet<char>,
    positive: bool,
}

fn parse_patterns(pattern: &str) -> Vec<Pattern> {
    let mut result: Vec<Pattern> = vec![];
    let mut pattern_characters = pattern.chars().peekable();

    while let Some(pattern_char) = pattern_characters.next() {
        match pattern_char {
            c if VALID_CHARACTERS.contains(&c) => {
                result.push(Pattern::BasicPattern(BasicPattern::Character(c)))
            }
            '\\' => {
                if let Some(next_char) = pattern_characters.next() {
                    match next_char {
                        c if VALID_ESCAPE_CHARACTERS.contains(&c) => {
                            result.push(Pattern::BasicPattern(BasicPattern::EscapeCharacter(c)))
                        }
                        c => panic!("Invalid escape character: {}", c),
                    }
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
                        c if VALID_CHARACTERS.contains(&c) => {
                            group_chars.insert(c);
                        }
                        c => panic!("Invalid character in character group: {}", c),
                    }
                }

                if !found_closing_bracket {
                    panic!("Character group never closed");
                }

                let group = CharacterGroup {
                    characters: group_chars,
                    positive: !is_negative_group,
                };

                result.push(Pattern::BasicPattern(BasicPattern::CharacterGroup(group)));
            }
            '^' => result.push(Pattern::BasicPattern(BasicPattern::BeginningOfLine)),
            '$' => result.push(Pattern::BasicPattern(BasicPattern::EndOfLine)),
            '*' => {
                if let Some(previous_pattern) = result.pop() {
                    match previous_pattern {
                        Pattern::BasicPattern(BasicPattern::BeginningOfLine) => {
                            panic!("Cannot repeat beginning of line",)
                        }
                        Pattern::BasicPattern(BasicPattern::EndOfLine) => {
                            panic!("Cannot repeat end of line",)
                        }
                        Pattern::BasicPattern(pattern) => result.push(Pattern::ZeroOrMore(pattern)),
                        _ => panic!("Cannot repeat pattern {:?}", previous_pattern),
                    }
                } else {
                    panic!("Cannot repeat at the beginning of the string");
                }
            }
            '+' => {
                if let Some(previous_pattern) = result.pop() {
                    match previous_pattern {
                        Pattern::BasicPattern(BasicPattern::BeginningOfLine) => {
                            panic!("Cannot repeat beginning of line",)
                        }
                        Pattern::BasicPattern(BasicPattern::EndOfLine) => {
                            panic!("Cannot repeat end of line",)
                        }
                        Pattern::BasicPattern(pattern) => {
                            result.push(Pattern::BasicPattern(pattern.clone()));
                            result.push(Pattern::ZeroOrMore(pattern));
                        }
                        _ => panic!("Cannot repeat pattern {:?}", previous_pattern),
                    }
                } else {
                    panic!("Cannot repeat at the beginning of the string");
                }
            }
            '.' => result.push(Pattern::BasicPattern(BasicPattern::Wildcard)),
            _ => panic!("Unhandled symbol: {}", pattern_char),
        }
    }

    result
}

fn match_patterns(input_characters: &mut PatternChars, patterns: &mut PatternsIter) -> bool {
    while let Some(pattern) = patterns.next() {
        let matched: bool = match pattern {
            Pattern::BasicPattern(pattern) => match_basic_pattern(input_characters, pattern),
            Pattern::ZeroOrMore(pattern) => {
                let mut stack = vec![input_characters.clone()];

                // Try to grab as many characters as possible, greedy style
                while match_basic_pattern(input_characters, pattern) {
                    stack.push(input_characters.clone());
                }

                // Work our way back down the stack until we match the rest of the input
                while let Some(mut remaining_input) = stack.pop() {
                    if match_patterns(&mut remaining_input, &mut patterns.clone()) {
                        return true;
                    }
                }

                false
            }
        };

        if !matched {
            return false;
        }
    }

    true
}

fn match_basic_pattern(input_characters: &mut PatternChars, pattern: &BasicPattern) -> bool {
    let next_char = input_characters.next();

    if next_char.is_none() {
        return matches!(pattern, BasicPattern::EndOfLine);
    }

    let next_char = next_char.unwrap();

    match pattern {
        BasicPattern::Character(c) => &next_char == c,
        BasicPattern::EscapeCharacter(c) => match_escape_pattern(next_char, c),
        BasicPattern::CharacterGroup(group) => match_character_group(next_char, group),
        BasicPattern::BeginningOfLine => false,
        BasicPattern::EndOfLine => next_char == '\n',
        BasicPattern::Wildcard => VALID_CHARACTERS.contains(&next_char),
    }
}

fn match_escape_pattern(next_char: char, escape_pattern: &char) -> bool {
    match escape_pattern {
        'd' => NUMERIC_CHARACTERS.contains(&next_char),
        'w' => ALPHANUMERIC_CHARACTERS.contains(&next_char),
        '\\' => next_char == '\\',
        _ => panic!("Unhandled escape pattern: {}", escape_pattern),
    }
}

fn match_character_group(next_char: char, group: &CharacterGroup) -> bool {
    match group.positive {
        true => group.characters.contains(&next_char),
        false => !group.characters.contains(&next_char),
    }
}
