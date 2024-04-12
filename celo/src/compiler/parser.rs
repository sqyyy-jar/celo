use std::rc::Rc;

use super::{
    error::{Error, Result},
    hir,
    lexer::Lexer,
    source::{Location, Source, Token, TokenKind},
    Compiler,
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
    UnknownMacro,
}

pub struct Parser<'a> {
    pub compiler: &'a Compiler,
    pub lexer: Lexer,
}

impl<'a> Parser<'a> {
    pub fn new(compiler: &'a Compiler, source: Rc<Source>) -> Self {
        Self {
            compiler,
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

    pub fn parse_module(&mut self, is_submodule: bool) -> Result<hir::Module> {
        let mut module = hir::Module::new(self.lexer.source());
        while self.lexer.peek_token()?.is_some() {
            let macro_token = self.expect_token(TokenKind::BangIdentifier)?.location;
            let macro_name = &module.source[macro_token];
            let Some(macro_handler) = self.compiler.get_macro(macro_name) else {
                return Err(self.make_error(Some(macro_token), ParserErrorKind::UnknownMacro));
            };
            (macro_handler)(self)?;
            // todo - improve
        }
        Ok(module)
    }

    /// Parses a group of nodes surrounded by curly braces.
    pub fn parse_scope(&mut self) -> Result<hir::Scope> {
        let start = self.expect_token(TokenKind::LeftCurly)?.location;
        let mut code = Vec::new();
        while let Some(node) = self.parse_node()? {
            code.push(node);
        }
        let end = self.expect_token(TokenKind::RightCurly)?.location;
        Ok(hir::Scope::new(start, end, code))
    }

    /// Parses a group of nodes surrounded by parentheses.
    pub fn parse_group(&mut self) -> Result<hir::Group> {
        let left_paren = self.expect_token(TokenKind::LeftParen)?.location;
        let mut nodes = Vec::new();
        while let Some(node) = self.parse_node()? {
            nodes.push(node);
        }
        let right_paren = self.expect_token(TokenKind::RightParen)?.location;
        Ok(hir::Group::new(left_paren, right_paren, nodes))
    }

    /// Parses a node.
    pub fn parse_node(&mut self) -> Result<Option<hir::Node>> {
        let Some(token) = self.lexer.peek_token()? else {
            return Ok(None);
        };
        let mut node = hir::Node::new(token.location, hir::NodeKind::Integer); // No kind set yet
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
                node = hir::Node::new(
                    group.left_paren.span_to(group.right_paren),
                    hir::NodeKind::Group(Box::new(group)),
                );
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
