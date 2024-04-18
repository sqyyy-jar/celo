use std::{fmt::Debug, rc::Rc};

use super::source::{Location, Source};

/// Represents the entire HIR structure of a compile task.
#[derive(Debug, Default)]
pub struct Hir {
    pub modules: Vec<Module>,
}

/// Represents a module or a submodule created by a macro.
#[derive(Debug)]
pub struct Module {
    pub source: Rc<Source>,
    pub submodules: Vec<usize>,
    pub functions: Vec<Function>,
}

impl Module {
    pub fn new(source: Rc<Source>) -> Self {
        Self {
            source,
            submodules: Vec::new(),
            functions: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub location: Location,
    pub name: Location,
    pub body: Scope,
    // todo
}

impl Function {
    pub fn new(location: Location, name: Location, body: Scope) -> Self {
        Self {
            location,
            name,
            body,
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    pub start: Location,
    pub end: Location,
    pub code: Vec<Node>,
}

impl Scope {
    pub fn new(start: Location, end: Location, code: Vec<Node>) -> Self {
        Self { start, end, code }
    }
}

#[derive(Debug)]
pub struct Node {
    pub location: Location,
    pub kind: NodeKind,
}

impl Node {
    pub fn new(location: Location, kind: NodeKind) -> Self {
        Self { location, kind }
    }
}

#[derive(Debug)]
pub enum NodeKind {
    Integer,
    Float,
    String,
    Call,
    Variable,
    Assignment(Box<Assignment>),
    Group(Box<Group>),
    MacroIntermediate(Box<dyn MacroIntermediate>),
}

#[derive(Debug)]
pub struct Assignment {
    pub arrow: Location,
    pub variable: Location,
}

impl Assignment {
    pub fn new(arrow: Location, variable: Location) -> Self {
        Self { arrow, variable }
    }
}

#[derive(Debug)]
pub struct Group {
    pub left_paren: Location,
    pub right_paren: Location,
    pub nodes: Vec<Node>,
}

impl Group {
    pub fn new(left_paren: Location, right_paren: Location, nodes: Vec<Node>) -> Self {
        Self {
            left_paren,
            right_paren,
            nodes,
        }
    }
}

// todo
pub trait MacroIntermediate: Debug {}
