use std::fs::File;
use std::io::{BufRead,BufReader};
use clap::Parser;

mod lib;
use crate::lib::errors::*;
use crate::lib::tokenizer::*;
use crate::lib::interpreter::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    filepath: String,

    #[clap(short, long)]
    debug: bool,
}

fn main() -> Result<(), CreateError> {
    let args = Args::parse();

    let tokens = match read_file(args.filepath.as_str()) {
        Err(e) => {
            if args.debug {println!("{:?}", e)} else {println!("{}", e)}
            return Err(CreateError { code: 2, message: "Could not read file properly.".to_string() });
        },
        Ok(d) => tokenize(d.as_str())?,
    };

    match interpret_program(tokens) {
        CreateResult::Ok() => (),
        CreateResult::Err(e) => {
            if args.debug {println!("{:?}", e)} else {println!("{}", e)}
            return Err(e);
        },
    }

    return Ok(());
}

fn read_file(file: &str) -> Result<String,CreateError> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|el| {el.expect("There was an unexpected issue.")}).fold(String::from(""), |mut acc,el| {
        acc.push('\n');
        acc.push_str(el.as_str());
        acc
    });
    return Ok(lines);
}
