use std::rc::Rc;

use super::source::{Location, Source};

#[derive(Debug)]
pub struct Mir {
    pub modules: Vec<Box<Module>>,
    pub functions: Vec<Box<Function>>,
}

#[derive(Debug)]
pub struct Module {
    pub source: Rc<Source>,
    pub submodules: Vec<usize>,
    pub functions: Vec<usize>,
}

#[derive(Debug)]
pub struct Function {
    pub location: Location,
    pub name: Location,
    pub body: Code,
}

#[derive(Debug)]
pub struct Code {} // todo - design code
