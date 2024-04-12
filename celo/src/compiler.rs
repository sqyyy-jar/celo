use std::{collections::HashMap, rc::Rc};

use self::{error::Result, parser::Parser, source::Source};

pub mod error;
pub mod hir;
pub mod lexer;
pub mod parser;
pub mod source;

pub type Macro = fn(&mut Parser) -> Result<()>;

pub struct Compiler {
    sources: Vec<Rc<Source>>,
    macros: HashMap<String, Macro>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            macros: HashMap::new(),
        }
    }

    pub fn add_source(&mut self, path: impl Into<Rc<str>>) -> Result<()> {
        self.sources.push(Rc::new(Source::load(path.into())?));
        Ok(())
    }

    pub fn get_macro(&self, name: &str) -> Option<Macro> {
        self.macros.get(name).copied()
    }

    pub fn compile(&mut self) -> Result<()> {
        let mut hir = hir::Hir::default();
        for source in &self.sources {
            let mut parser = Parser::new(self, source.clone());
            let module = parser.parse_module(false)?;
            hir.modules.push(module);
        }
        Ok(())
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

pub mod experimental {
    use super::{error::Result, parser::Parser, source::TokenKind, Compiler};

    pub fn init(compiler: &mut Compiler) {
        compiler.macros.insert("fn!".to_owned(), macro_fn);
    }

    fn macro_fn(parser: &mut Parser) -> Result<()> {
        let name = parser.expect_token(TokenKind::Identifier)?.location;
        let scope = parser.parse_scope()?;
        println!("fn {name:?} {scope:#?}");
        Ok(())
    }
}
