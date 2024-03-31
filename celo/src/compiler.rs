use std::rc::Rc;

use self::{error::Result, parser::Parser, source::Source};

pub mod error;
pub mod hir;
pub mod lexer;
pub mod parser;
pub mod source;

pub struct Compiler {
    sources: Vec<Rc<Source>>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add_source(&mut self, path: impl Into<Rc<str>>) -> Result<()> {
        self.sources.push(Rc::new(Source::load(path.into())?));
        Ok(())
    }

    pub fn compile(&mut self) -> Result<()> {
        for source in &self.sources {
            let mut parser = Parser::new(source.clone());
            let scope = parser.parse_scope(None, None)?;
            eprintln!("scope {scope:?}");
        }
        Ok(())
    }
}
