use std::fs::File;
use std::io::{BufRead,BufReader};
use clap::Parser;

mod lib;
use crate::lib::errors::*;
use crate::lib::tokenizer::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    filepath: String,

    #[clap(short, long)]
    debug: bool,
}

fn main() -> Result<(), CompilerError> {
    let args = Args::parse();
    println!("{:?}", args);

    let tokens = match read_file(args.filepath.as_str()) {
        Err(e) => {
            if args.debug {println!("{:?}", e)} else {println!("{}", e)}
            return Err(CompilerError { code: 2, message: "Could not read file properly.".to_string() });
        },
        Ok(d) => tokenize(d.as_str()),
    };

    return Ok(());
}

fn read_file(file: &str) -> Result<String,CompilerError> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|el| {el.expect("There was an unexpected issue.")}).fold(String::from(""), |mut acc,el| {
        acc.push('\n');
        acc.push_str(el.as_str());
        acc
    });
    return Ok(lines);
}
