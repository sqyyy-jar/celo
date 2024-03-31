use super::source::Location;

#[derive(Debug)]
pub struct Scope {
    pub parent: Option<usize>,
    pub name: Option<Location>,
    pub start: Option<Location>,
    pub end: Option<Location>,
    pub code: Vec<Node>,
    pub children: Vec<(Location, usize)>,
}

impl Scope {
    pub fn new(parent: Option<usize>, name: Option<Location>) -> Self {
        Self {
            parent,
            name,
            start: None,
            end: None,
            code: Vec::new(),
            children: Vec::new(),
        }
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
    MacroIntermediate(Box<()>), // todo
}

#[derive(Debug)]
pub struct Assignment {
    pub arrow: Location,
    pub variable: Location,
}
