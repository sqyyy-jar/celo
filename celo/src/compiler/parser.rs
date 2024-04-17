use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

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

pub type MacroHandler = fn(&mut ParseHirStep) -> Result<()>;

pub struct ParseHirStep<'a> {
    pub compiler: &'a Compiler,
    pub lexer: Lexer,
    root_scope: MacroScope,
    macro_scopes: Vec<MacroScope>,
    hir: hir::Hir,
    current_module: usize,
    /// Queue of modules that are yet to be parsed
    source_queue: VecDeque<usize>,
    // todo
}

impl<'a> ParseHirStep<'a> {
    pub fn new(compiler: &'a Compiler, main_source: Rc<Source>) -> Self {
        Self {
            compiler,
            lexer: Lexer::new(main_source),
            root_scope: MacroScope::default(),
            macro_scopes: Vec::new(),
            hir: hir::Hir::default(),
            current_module: 0,
            source_queue: VecDeque::new(),
        }
    }

    pub fn add_root_macro(&mut self, name: impl Into<String>, handler: MacroHandler) {
        self.root_scope.add(name.into(), handler);
    }

    pub fn add_local_macro(&mut self, name: impl Into<String>, handler: MacroHandler) {
        self.macro_scopes
            .last_mut()
            .unwrap()
            .add(name.into(), handler);
    }

    pub fn resolve_macro(&self, name: &str) -> Option<MacroHandler> {
        // root-level macros cannot be overridden
        if let Some(handler) = self.root_scope.resolve(name) {
            return Some(handler);
        }
        for scope in self.macro_scopes.iter().rev() {
            if let Some(handler) = scope.resolve(name) {
                return Some(handler);
            }
        }
        None
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

    pub fn parse_module(&mut self, is_submodule: bool) -> Result<usize> {
        let module_index = self.hir.modules.len();
        let previous_module_index = self.current_module;
        self.current_module = module_index;
        self.hir.modules.push(hir::Module::new(self.lexer.source()));
        self.macro_scopes.push(MacroScope::default());
        let source = self.lexer.source();
        loop {
            let Some(token) = self.lexer.peek_token()? else {
                break;
            };
            if token.kind != TokenKind::BangIdentifier {
                break;
            }
            let macro_token = self.expect_token(TokenKind::BangIdentifier)?.location;
            let macro_name = &source[macro_token].trim_end_matches('!');
            let Some(macro_handler) = self.resolve_macro(macro_name) else {
                return Err(self.make_error(Some(macro_token), ParserErrorKind::UnknownMacro));
            };
            (macro_handler)(self)?;
        }
        self.current_module = previous_module_index;
        self.macro_scopes.pop().unwrap();
        Ok(module_index)
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

    pub fn run(mut self) -> Result<hir::Hir> {
        _ = self.parse_module(false)?;
        while let Some(module_index) = self.source_queue.pop_front() {
            self.current_module = module_index;
            self.parse_module(false)?;
        }
        Ok(self.hir)
    }
}

#[derive(Default)]
pub struct MacroScope {
    handlers: HashMap<String, MacroHandler>,
}

impl MacroScope {
    pub fn add(&mut self, name: impl Into<String>, handler: MacroHandler) {
        self.handlers.insert(name.into(), handler);
    }

    pub fn resolve(&self, name: &str) -> Option<MacroHandler> {
        self.handlers.get(name).copied()
    }
}
