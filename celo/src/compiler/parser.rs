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

    pub fn make_error(&mut self, location: Option<Location>, kind: ParserErrorKind) -> Error {
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
        self.lexer.consume_token()?;
        Ok(token)
    }

    /// Parses a group of nodes surrounded by curly braces
    pub fn parse_scope(&mut self, is_root: bool) -> Result<hir::Scope> {
        let mut scope = hir::Scope::default();
        if !is_root {
            scope.start = Some(self.expect_token(TokenKind::LeftCurly)?.location);
        }
        while let Some(node) = self.parse_node()? {
            scope.code.push(node);
        }
        if !is_root {
            scope.end = Some(self.expect_token(TokenKind::RightCurly)?.location);
        }
        Ok(scope)
    }

    /// Parses a group of nodes surrounded by parentheses
    pub fn parse_group(&mut self) -> Result<hir::Group> {
        let left_paren = self.expect_token(TokenKind::LeftParen)?.location;
        let mut nodes = Vec::new();
        while let Some(node) = self.parse_node()? {
            nodes.push(node);
        }
        let right_paren = self.expect_token(TokenKind::RightParen)?.location;
        Ok(hir::Group {
            left_paren,
            right_paren,
            nodes,
        })
    }

    /// Parses a node
    pub fn parse_node(&mut self) -> Result<Option<hir::Node>> {
        let Some(token) = self.lexer.peek_token()? else {
            return Ok(None);
        };
        let mut node = hir::Node {
            location: token.location,
            kind: hir::NodeKind::Integer, // No kind set yet
        };
        match token.kind {
            TokenKind::Integer => {
                self.lexer.consume_token()?;
                node.kind = hir::NodeKind::Integer;
            }
            TokenKind::Float => {
                self.lexer.consume_token()?;
                node.kind = hir::NodeKind::Float;
            }
            TokenKind::String => {
                self.lexer.consume_token()?;
                node.kind = hir::NodeKind::String;
            }
            TokenKind::LeftParen => {
                let group = self.parse_group()?;
                node = hir::Node {
                    location: group.left_paren.span_to(group.right_paren),
                    kind: hir::NodeKind::Group(Box::new(group)),
                };
            }
            TokenKind::RightParen => return Ok(None),
            TokenKind::LeftSquare => unimplemented!("square-brackets?"),
            TokenKind::RightSquare => return Ok(None),
            TokenKind::LeftCurly => unimplemented!("scopes?"),
            TokenKind::RightCurly => return Ok(None),
            TokenKind::Identifier => {
                self.lexer.consume_token()?;
                node.kind = hir::NodeKind::Call;
            }
            TokenKind::DotIdentifier => {
                self.lexer.consume_token()?;
                node.kind = hir::NodeKind::Variable;
            }
            TokenKind::BangIdentifier => unimplemented!("macros"),
            TokenKind::RightArrow => node = self.parse_assignment()?,
        }
        Ok(Some(node))
    }

    fn parse_assignment(&mut self) -> Result<hir::Node> {
        let arrow = self.expect_token(TokenKind::RightArrow)?.location;
        let variable = self.expect_token(TokenKind::DotIdentifier)?.location;
        Ok(hir::Node {
            location: arrow.span_to(variable),
            kind: hir::NodeKind::Assignment(Box::new(hir::Assignment { arrow, variable })),
        })
    }
}
