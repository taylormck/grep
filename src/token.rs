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
    OpenBracket,
    CloseBracket,
    Dot,
    Caret,
}

pub fn tokenize(str: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let characters = str.chars().peekable();

    for c in characters {
        let new_token = match c {
            '.' => Token::Dot,
            '\\' => Token::BackSlash,
            '(' => Token::OpenParen,
            ')' => Token::CloseParen,
            '[' => Token::OpenBracket,
            ']' => Token::CloseBracket,
            '^' => Token::Caret,
            '*' => Token::Star,
            '+' => Token::Plus,
            '|' => Token::Pipe,
            '-' => Token::Hyphen,
            ' ' => Token::Space,
            c if character_sets::ALPHANUMERIC_CHARACTERS.contains(&c) => Token::Literal(c),
            _ => panic!("Unhandled character: {}", c),
        };

        tokens.push(new_token);
    }

    tokens
}
