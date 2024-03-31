use std::rc::Rc;

use phf::{phf_map, Map};

use super::{
    error::{Error, Result},
    source::{Location, Source, Token, TokenKind},
};

pub const KEYWORDS: Map<&str, TokenKind> = phf_map! {
    "->" => TokenKind::RightArrow,
};

#[derive(Debug)]
pub struct LexerError {
    source: Rc<Source>,
    location: Location,
    kind: LexerErrorKind,
}

#[derive(Debug)]
pub enum LexerErrorKind {
    InvalidCharacter,
    InvalidEof,
    InvalidEscapeSequence,
}

pub struct Lexer {
    source: Rc<Source>,
    start: u32,
    start_line: u32,
    start_column: u32,
    current: u32,
    current_line: u32,
    current_column: u32,
    peek_buf: Option<Token>,
}

impl Lexer {
    pub fn new(source: Rc<Source>) -> Self {
        Self {
            source,
            start: 0,
            start_line: 1,
            start_column: 1,
            current: 0,
            current_line: 1,
            current_column: 1,
            peek_buf: None,
        }
    }

    pub fn source(&self) -> Rc<Source> {
        self.source.clone()
    }

    fn peek(&self) -> Option<char> {
        if self.current as usize >= self.source.content.len() {
            return None;
        }
        self.source.content[self.current as usize..].chars().next()
    }

    fn next(&mut self) {
        let Some(c) = self.peek() else {
            return;
        };
        self.current += c.len_utf8() as u32;
        self.current_column += 1;
        if c == '\n' {
            self.current_line += 1;
            self.current_column = 1;
        }
    }

    fn clear_location(&mut self) {
        self.start = self.current;
        self.start_line = self.current_line;
        self.start_column = self.current_column;
    }

    fn make_location(&mut self) -> Location {
        let location = Location {
            start: self.start,
            end: self.current,
            line: self.start_line,
            column: self.start_column,
        };
        self.clear_location();
        location
    }

    fn make_token(&mut self, kind: TokenKind) -> Token {
        Token {
            location: self.make_location(),
            kind,
        }
    }

    fn make_error(&mut self, kind: LexerErrorKind) -> Error {
        Error::Lexer(Box::new(LexerError {
            source: self.source.clone(),
            location: self.make_location(),
            kind,
        }))
    }

    fn skip_comment(&mut self) {
        while let Some(c) = self.peek() {
            self.next();
            if c == '\n' {
                break;
            }
        }
    }

    fn parse_minus_prefix(&mut self) -> Result<Token> {
        let Some(c) = self.peek() else {
            return self.parse_identifier(false);
        };
        if c.is_ascii_digit() {
            return self.parse_number();
        }
        self.parse_identifier(false)
    }

    fn parse_identifier(&mut self, dot: bool) -> Result<Token> {
        let mut first = dot;
        let mut bang = false;
        while let Some(c) = self.peek() {
            if c == '.' {
                self.next();
                return Err(self.make_error(LexerErrorKind::InvalidCharacter));
            }
            if bang {
                if c == '!' || is_identifier(c, false) {
                    self.next();
                    return Err(self.make_error(LexerErrorKind::InvalidCharacter));
                }
                break;
            }
            if is_identifier(c, first) {
                self.next();
                first = false;
                continue;
            }
            if c == '!' {
                self.next();
                if dot {
                    return Err(self.make_error(LexerErrorKind::InvalidCharacter));
                }
                bang = true;
                continue;
            }
            break;
        }
        if dot {
            return Ok(self.make_token(TokenKind::DotIdentifier));
        }
        if bang {
            return Ok(self.make_token(TokenKind::BangIdentifier));
        }
        let mut token = self.make_token(TokenKind::Identifier);
        let string =
            &self.source.content[token.location.start as usize..token.location.end as usize];
        if let Some(&kind) = KEYWORDS.get(string) {
            token.kind = kind;
        }
        Ok(token)
    }

    fn parse_number(&mut self) -> Result<Token> {
        let mut dot = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.next();
                continue;
            }
            if c == '.' {
                self.next();
                if dot {
                    return Err(self.make_error(LexerErrorKind::InvalidCharacter));
                }
                dot = true;
                continue;
            }
            break;
        }
        if dot {
            return Ok(self.make_token(TokenKind::Float));
        }
        Ok(self.make_token(TokenKind::Integer))
    }

    fn parse_string(&mut self) -> Result<Token> {
        loop {
            let Some(c) = self.peek() else {
                return Err(self.make_error(LexerErrorKind::InvalidEof));
            };
            self.next();
            if c == '"' {
                break;
            }
            if c != '\\' {
                continue;
            }
            // Escape sequences
            let Some(escape) = self.peek() else {
                return Err(self.make_error(LexerErrorKind::InvalidEof));
            };
            self.next();
            match escape {
                '"' | '\\' | 'n' | 'r' | 't' => (),
                _ => return Err(self.make_error(LexerErrorKind::InvalidEscapeSequence)),
            }
        }
        Ok(self.make_token(TokenKind::String))
    }

    fn parse_token(&mut self) -> Result<Option<Token>> {
        loop {
            let Some(c) = self.peek() else {
                break Ok(None);
            };
            self.next();
            if c.is_ascii_whitespace() {
                self.clear_location();
                continue;
            }
            match c {
                ';' => self.skip_comment(),
                '(' => break Ok(Some(self.make_token(TokenKind::LeftParen))),
                ')' => break Ok(Some(self.make_token(TokenKind::RightParen))),
                '[' => break Ok(Some(self.make_token(TokenKind::LeftSquare))),
                ']' => break Ok(Some(self.make_token(TokenKind::RightSquare))),
                '{' => break Ok(Some(self.make_token(TokenKind::LeftCurly))),
                '}' => break Ok(Some(self.make_token(TokenKind::RightCurly))),
                '-' => break self.parse_minus_prefix().map(Some),
                '.' => break self.parse_identifier(true).map(Some),
                '"' => break self.parse_string().map(Some),
                '0'..='9' => break self.parse_number().map(Some),
                _ => {
                    if is_identifier(c, true) {
                        break self.parse_identifier(false).map(Some);
                    }
                    break Err(self.make_error(LexerErrorKind::InvalidCharacter));
                }
            }
        }
    }

    pub fn peek_token(&mut self) -> Result<Option<Token>> {
        if self.peek_buf.is_some() {
            return Ok(self.peek_buf);
        }
        self.peek_buf = self.parse_token()?;
        Ok(self.peek_buf)
    }

    pub fn consume_token(&mut self) -> Result<()> {
        if self.peek_buf.take().is_none() {
            self.parse_token()?;
        }
        Ok(())
    }
}

fn is_identifier(c: char, first: bool) -> bool {
    if c.is_alphabetic() {
        return true;
    }
    if !first && c.is_ascii_digit() {
        return true;
    }
    if "+-*/%=<>&|^_:?#@~".contains(c) {
        return true;
    }
    false
}
