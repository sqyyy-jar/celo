use std::{env::args, process::exit};

fn main() {
    let args: Vec<String> = args().skip(1).collect();
    exit(celo_cli::main(&args));
}
