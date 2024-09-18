use crate::character_sets;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    Literal(char),
    BackSlash,
    OpenParen,
    CloseParen,
    Star,
    Plus,
    Pipe,
    Hyphen,
    Space,
}

pub fn tokenize(str: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let characters = str.chars().peekable();

    for c in characters {
        let new_token = match c {
            c if character_sets::ALPHANUMERIC_CHARACTERS.contains(&c) => Token::Literal(c),
            '\\' => Token::BackSlash,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            '*' => Token::Star,
            '+' => Token::Plus,
            '|' => Token::Pipe,
            '-' => Token::Hyphen,
            ' ' => Token::Space,
            _ => panic!("Unhandled character: {}", c),
        };

        tokens.push(new_token);
    }

    tokens
}
