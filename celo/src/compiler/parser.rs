use std::rc::Rc;

use super::{
    error::{Error, Result},
    hir,
    lexer::Lexer,
    source::{Location, Source, Token, TokenKind},
};

#[derive(Debug)]
pub struct ParserError {
    source: Rc<Source>,
    location: Option<Location>,
    kind: ParserErrorKind,
}

#[derive(Debug)]
pub enum ParserErrorKind {
    UnclosedScope,
    UnexpectedToken {
        expected: Option<TokenKind>,
        got: TokenKind,
    },
    UnexpectedEof {
        expected: Option<TokenKind>,
    },
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(source: Rc<Source>) -> Self {
        Self {
            lexer: Lexer::new(source),
        }
    }

    fn make_error(&mut self, location: Option<Location>, kind: ParserErrorKind) -> Error {
        Error::Parser(Box::new(ParserError {
            source: self.lexer.source(),
            location,
            kind,
        }))
    }

    pub fn expect_token(&mut self, kind: TokenKind) -> Result<Token> {
        let Some(token) = self.lexer.peek_token()? else {
            return Err(self.make_error(
                None,
                ParserErrorKind::UnexpectedEof {
                    expected: Some(kind),
                },
            ));
        };
        if token.kind != kind {
            return Err(self.make_error(
                Some(token.location),
                ParserErrorKind::UnexpectedToken {
                    expected: Some(kind),
                    got: token.kind,
                },
            ));
        }
        Ok(token)
    }

    pub fn parse_scope(
        &mut self,
        parent: Option<usize>,
        name: Option<Location>,
    ) -> Result<hir::Scope> {
        let is_root = parent.is_none();
        let mut scope = hir::Scope::new(parent, name);
        scope.start = if !is_root {
            Some(self.expect_token(TokenKind::LeftCurly)?.location)
        } else {
            None
        };
        loop {
            let Some(token) = self.lexer.peek_token()? else {
                if !is_root {
                    return Err(self.make_error(scope.start, ParserErrorKind::UnclosedScope));
                }
                break;
            };
            self.lexer.consume_token()?;
            let mut node = hir::Node {
                location: token.location,
                kind: hir::NodeKind::Integer, // None
            };
            match token.kind {
                TokenKind::Integer => node.kind = hir::NodeKind::Integer,
                TokenKind::Float => node.kind = hir::NodeKind::Float,
                TokenKind::String => node.kind = hir::NodeKind::String,
                TokenKind::LeftParen => todo!(),
                TokenKind::RightParen => todo!(),
                TokenKind::LeftCurly => todo!(),
                TokenKind::RightCurly if !is_root => {
                    scope.end = Some(token.location);
                    break;
                }
                TokenKind::Identifier => {
                    // todo - child scopes
                    node.kind = hir::NodeKind::Call
                }
                TokenKind::DotIdentifier => node.kind = hir::NodeKind::Variable,
                TokenKind::BangIdentifier => todo!(),
                TokenKind::RightArrow => todo!(),
                _ => {
                    return Err(self.make_error(
                        Some(token.location),
                        ParserErrorKind::UnexpectedToken {
                            expected: None,
                            got: token.kind,
                        },
                    ));
                }
            }
            eprintln!("node {node:?}");
            scope.code.push(node);
        }
        Ok(scope)
    }
}
