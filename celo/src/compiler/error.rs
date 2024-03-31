use super::{lexer::LexerError, parser::ParserError, source::SourceError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Source(Box<SourceError>),
    Lexer(Box<LexerError>),
    Parser(Box<ParserError>),
}
