use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path;
use std::process;

type TWResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> TWResult<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => run_prompt(),
        1 => run_file(&args[0]),
        _ => {
            println!("Usage: tree-walk [script]");
            process::exit(64);
        }
    }
}

fn run_prompt() -> TWResult<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        if line.len() == 0 {
            break;
        }

        run(line)?;
    }

    Ok(())
}

fn run_file<P: AsRef<path::Path> + fmt::Display>(filename: P) -> TWResult<()> {
    run(fs::read_to_string(filename)?)
}

fn run(source: String) -> TWResult<()> {
    for token in source.split(" ") {
        println!("{}", token);
    }

    Ok(())
}
