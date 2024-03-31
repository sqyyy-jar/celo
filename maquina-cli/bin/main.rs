use std::{env, process::exit};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    exit(maquina_cli::main(&args));
}
