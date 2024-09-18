use core::{slice, str};
use phf::phf_set;
use std::{boxed::Box, collections::HashSet, env, io, iter, process};

type PatternChars<'a> = iter::Peekable<str::Chars<'a>>;
type PatternsIter<'a> = slice::Iter<'a, Pattern>;

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
    '5', '6', '7', '8', '9', '_', '-', ' ', '*', '+', '|',
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
    let tokenized_patterns = tokenize_search_pattern(&pattern);

    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();

    let mut input_chars = input_line.chars().peekable();
    let mut back_refs = Vec::<String>::new();

    dbg!(&tokenized_patterns);

    match tokenized_patterns[0] {
        Pattern::Basic(BasicPattern::BeginningOfLine) => {
            if match_patterns(
                &mut input_chars.clone(),
                &mut tokenized_patterns[1..].iter(),
                &mut back_refs,
            ) {
                process::exit(0);
            }
        }
        _ => {
            while input_chars.peek().is_some() {
                if match_patterns(
                    &mut input_chars.clone(),
                    &mut tokenized_patterns.iter(),
                    &mut back_refs,
                ) {
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
    Basic(BasicPattern),
    ZeroOrMore(Box<Pattern>),
    Sequence(Vec<Pattern>),
    Union(Vec<Pattern>),
    Backreference(usize),
    Empty,
}

#[derive(Clone, Debug)]
struct CharacterGroup {
    characters: HashSet<char>,
    positive: bool,
}

fn tokenize_search_pattern(pattern: &str) -> Vec<Pattern> {
    let mut result: Vec<Pattern> = vec![];
    let mut pattern_characters = pattern.chars().peekable();

    while let Some(pattern_char) = pattern_characters.next() {
        match pattern_char {
            '\\' => {
                // First, check for a backreference number
                let mut digits = Vec::<char>::new();
                while let Some(digit) = pattern_characters.peek() {
                    if NUMERIC_CHARACTERS.contains(digit) {
                        digits.push(*digit);
                        pattern_characters.next();
                    } else {
                        break;
                    }
                }

                if !digits.is_empty() {
                    if let Ok(number) = digits.iter().collect::<String>().parse::<usize>() {
                        result.push(Pattern::Backreference(number));
                        continue;
                    }
                }

                // Not a number, so check for escape characters
                if let Some(next_char) = pattern_characters.next() {
                    match next_char {
                        c if VALID_ESCAPE_CHARACTERS.contains(&c) => {
                            result.push(Pattern::Basic(BasicPattern::EscapeCharacter(c)))
                        }
                        c => panic!("Invalid escape character: {}", c),
                    }
                }
            }
            '[' => {
                let is_negative_group = *pattern_characters.peek().unwrap() == '^';

                if is_negative_group {
                    pattern_characters.next();
                }

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

                result.push(Pattern::Basic(BasicPattern::CharacterGroup(group)));
            }
            '(' => {
                let mut chars_so_far = Vec::<char>::new();
                let mut found_closing_bracket = false;
                let mut num_opening_brackets = 0;
                let mut sequences = Vec::<Pattern>::new();

                let mut push_sequence = |chars: &str| {
                    let pattern = match !chars.contains("(") && chars.contains("|") {
                        true => Pattern::Union(
                            chars
                                .split("|")
                                .map(tokenize_search_pattern)
                                .map(Pattern::Sequence)
                                .collect::<Vec<Pattern>>(),
                        ),
                        false => Pattern::Sequence(tokenize_search_pattern(chars)),
                    };

                    sequences.push(pattern);
                };

                while let Some(group_char) = pattern_characters.peek() {
                    match group_char {
                        '(' => {
                            if num_opening_brackets == 0 {
                                let chars = chars_so_far.iter().collect::<String>();
                                push_sequence(chars.as_str());
                                chars_so_far.clear();
                            }
                            num_opening_brackets += 1;
                            chars_so_far.push('(');
                        }
                        ')' => {
                            if num_opening_brackets == 0 {
                                found_closing_bracket = true;

                                let chars = chars_so_far.iter().collect::<String>();
                                push_sequence(chars.as_str());

                                pattern_characters.next();
                                break;
                            } else {
                                chars_so_far.push(')');
                                num_opening_brackets -= 1;
                            }
                        }
                        c if VALID_CHARACTERS.contains(c) => {
                            chars_so_far.push(*c);
                        }
                        c => panic!("Invalid character in character group: {}", c),
                    }
                    pattern_characters.next();
                }

                if !found_closing_bracket {
                    panic!("Character group never closed");
                }

                result.extend(sequences);
            }
            '^' => result.push(Pattern::Basic(BasicPattern::BeginningOfLine)),
            '$' => result.push(Pattern::Basic(BasicPattern::EndOfLine)),
            '*' => {
                if let Some(previous_pattern) = result.pop() {
                    match previous_pattern {
                        Pattern::Basic(BasicPattern::BeginningOfLine) => {
                            panic!("Cannot repeat beginning of line",)
                        }
                        Pattern::Basic(BasicPattern::EndOfLine) => {
                            panic!("Cannot repeat end of line",)
                        }
                        Pattern::Union(_) => panic!("Repeating unions is not yet supported"),
                        Pattern::ZeroOrMore(_) => panic!("Cannot repeat recursively"),
                        pattern => result.push(Pattern::ZeroOrMore(Box::new(pattern))),
                    }
                } else {
                    panic!("Cannot repeat at the beginning of the string");
                }
            }
            '+' => {
                if let Some(previous_pattern) = result.pop() {
                    match previous_pattern {
                        Pattern::Basic(BasicPattern::BeginningOfLine) => {
                            panic!("Cannot repeat beginning of line",)
                        }
                        Pattern::Basic(BasicPattern::EndOfLine) => {
                            panic!("Cannot repeat end of line",)
                        }
                        Pattern::Union(_) => panic!("Repeating unions is not yet supported"),
                        Pattern::ZeroOrMore(_) => panic!("Cannot repeat recursively"),
                        pattern => {
                            result.push(pattern.clone());
                            result.push(Pattern::ZeroOrMore(Box::new(pattern)));
                        }
                    }
                } else {
                    panic!("Cannot repeat at the beginning of the string");
                }
            }
            '?' => {
                if let Some(previous_pattern) = result.pop() {
                    match previous_pattern {
                        Pattern::Basic(BasicPattern::BeginningOfLine) => {
                            panic!("Cannot make beginning of line optional",)
                        }
                        Pattern::Basic(BasicPattern::EndOfLine) => {
                            panic!("Cannot make end of line optional",)
                        }
                        Pattern::ZeroOrMore(_) => panic!("Cannot repeat recursively"),
                        pattern => {
                            result.push(Pattern::Union(vec![pattern, Pattern::Empty]));
                        }
                    }
                } else {
                    panic!("Cannot repeat at the beginning of the string");
                }
            }
            '.' => result.push(Pattern::Basic(BasicPattern::Wildcard)),
            c if VALID_CHARACTERS.contains(&c) => {
                result.push(Pattern::Basic(BasicPattern::Character(c)))
            }
            _ => panic!("Unhandled symbol: {}", pattern_char),
        }
    }

    result
}

fn match_patterns(
    input_characters: &mut PatternChars,
    patterns: &mut PatternsIter,
    back_refs: &mut Vec<String>,
) -> bool {
    while let Some(pattern) = patterns.next() {
        dbg!(pattern);
        let matched: bool = match pattern {
            Pattern::Empty => true,
            Pattern::Basic(pattern) => match_basic_pattern(input_characters, pattern),
            Pattern::ZeroOrMore(pattern) => {
                let mut stack = vec![input_characters.clone()];
                let pattern = [*pattern.clone()];

                // Try to grab as many characters as possible, greedy style
                while match_patterns(
                    input_characters,
                    &mut pattern.iter(),
                    &mut back_refs.clone(),
                ) {
                    stack.push(input_characters.clone());
                }

                // Work our way back down the stack until we match the rest of the input
                while let Some(mut remaining_input) = stack.pop() {
                    if match_patterns(
                        &mut remaining_input,
                        &mut patterns.clone(),
                        &mut back_refs.clone(),
                    ) {
                        //  Empty the iterator
                        for _ in patterns {}

                        return true;
                    }
                }

                false
            }
            Pattern::Sequence(patterns) => {
                let mut copy = input_characters.clone();

                let s1: String = input_characters.clone().collect();

                if match_patterns(input_characters, &mut patterns.iter(), back_refs) {
                    let s2: String = input_characters.clone().collect();
                    dbg!(s1, s2);

                    let mut back_ref = String::new();

                    while copy.clone().lt(&mut *input_characters) {
                        back_ref.push(copy.next().unwrap());
                    }

                    back_ref.pop();

                    back_refs.push(back_ref);

                    return true;
                }

                false
            }
            Pattern::Union(union_patterns) => union_patterns.iter().any(|sub_pattern| {
                let mut remaining_patterns = vec![sub_pattern.clone()];
                remaining_patterns.extend(patterns.clone().cloned());

                let mut input_characters_clone = input_characters.clone();

                if match_patterns(
                    &mut input_characters_clone,
                    &mut remaining_patterns.as_slice().iter(),
                    &mut back_refs.clone(),
                ) {
                    //  Empty the iterator
                    for _ in &mut *patterns {}
                    return true;
                }

                false
            }),
            Pattern::Backreference(back_ref_index) => {
                dbg!(back_ref_index);
                if *back_ref_index < back_refs.len() {
                    let pattern: Vec<Pattern> = back_refs[*back_ref_index]
                        .chars()
                        .map(BasicPattern::Character)
                        .map(Pattern::Basic)
                        .collect();

                    return match_patterns(input_characters, &mut pattern.iter(), back_refs);
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
