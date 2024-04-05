use super::source::Location;

#[derive(Debug)]
pub struct Scope {
    pub start: Option<Location>,
    pub end: Option<Location>,
    pub code: Vec<Node>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            code: Vec::new(),
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Node {
    pub location: Location,
    pub kind: NodeKind,
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
    MacroIntermediate(Box<()>), // todo
}

#[derive(Debug)]
pub struct Assignment {
    pub arrow: Location,
    pub variable: Location,
}

#[derive(Debug)]
pub struct Group {
    pub left_paren: Location,
    pub right_paren: Location,
    pub nodes: Vec<Node>,
}
