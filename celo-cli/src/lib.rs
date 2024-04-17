use celo::compiler::{source::Source, Compiler};

pub fn main(args: &[String]) -> i32 {
    let main_source = Source::load(args[0].as_str()).expect("load source");
    let mut compiler = Compiler::new(main_source);
    compiler.compile().expect("compile program");
    0
}
