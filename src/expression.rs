use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Empty,
    Sequence(Vec<Expression>),
    Literal(char),
    Escape(char),
    Capture(Box<Expression>),
    CharacterGroup(HashSet<char>),
    NegativeCharacterGroup(HashSet<char>),
    BeginningOfLine,
    EndOfLine,
    Repeat(Box<Expression>),
    Optional(Box<Expression>),
}
