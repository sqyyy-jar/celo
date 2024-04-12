use celo::compiler::Compiler;

pub fn main(args: &[String]) -> i32 {
    let mut compiler = Compiler::default();
    celo::compiler::experimental::init(&mut compiler);
    for arg in args {
        compiler.add_source(arg.to_owned()).expect("load source");
    }
    compiler.compile().expect("compile program");
    0
}
