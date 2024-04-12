use std::{io, ops::Index, rc::Rc};

use super::error::{Error, Result};

#[derive(Debug)]
pub struct SourceError {
    path: Rc<str>,
    kind: SourceErrorKind,
}
#[derive(Debug)]

pub enum SourceErrorKind {
    FileNotFound,
    PermissionDenied,
    IoError(std::io::Error),
}

#[derive(Debug)]
pub struct Source {
    pub path: Rc<str>,
    pub content: Rc<str>,
}

impl Source {
    pub fn load(path: impl Into<Rc<str>>) -> Result<Self> {
        let path = path.into();
        match std::fs::read_to_string(&*path) {
            Ok(content) => Ok(Self {
                path,
                content: content.into(),
            }),
            Err(err) => Err(Error::Source(Box::new(SourceError {
                path,
                kind: match err.kind() {
                    io::ErrorKind::NotFound => SourceErrorKind::FileNotFound,
                    io::ErrorKind::PermissionDenied => SourceErrorKind::PermissionDenied,
                    _ => SourceErrorKind::IoError(err),
                },
            }))),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Location {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

impl Location {
    pub fn span_to(self, other: Self) -> Self {
        Self {
            start: self.start,
            end: other.end,
            line: self.line,
            column: self.column,
        }
    }
}

impl Index<Location> for Source {
    type Output = str;

    fn index(&self, index: Location) -> &Self::Output {
        &self.content[index.start as usize..index.end as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub location: Location,
    pub kind: TokenKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    Integer,
    Float,
    String,
    // Brackets
    LeftParen,
    RightParen,
    LeftSquare,
    RightSquare,
    LeftCurly,
    RightCurly,
    // Identifiers
    Identifier,
    DotIdentifier,
    BangIdentifier,
    // Keywords
    RightArrow,
}
