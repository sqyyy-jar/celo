use std::rc::Rc;

use self::{error::Result, parser::ParseHirStep, source::Source};

pub mod error;
pub mod hir;
pub mod lexer;
pub mod mir;
pub mod parser;
pub mod source;

pub struct Compiler {
    main_source: Rc<Source>,
}

impl Compiler {
    pub fn new(main_source: Rc<Source>) -> Self {
        Self { main_source }
    }

    pub fn compile(&mut self) -> Result<()> {
        let mut hir_step = ParseHirStep::new(self, self.main_source.clone());
        experimental::init(&mut hir_step);
        let hir = hir_step.run()?;
        println!("{hir:#?}");
        // todo - implement
        Ok(())
    }
}

pub mod experimental {
    use super::{error::Result, hir, parser::ParseHirStep, source::TokenKind};

    pub fn init(step: &mut ParseHirStep) {
        step.add_root_macro("fn", macro_fn);
    }

    fn macro_fn(step: &mut ParseHirStep) -> Result<()> {
        let name = step.expect_token(TokenKind::Identifier)?.location;
        let scope = step.parse_scope()?;
        step.add_function(hir::Function::new(name.span_to(scope.end), name, scope));
        Ok(())
    }
}
