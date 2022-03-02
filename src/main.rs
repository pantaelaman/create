use std::fs::File;
use std::io::{BufRead,BufReader};
use clap::Parser;

mod lib;
use crate::lib::errors::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    filepath: String,

    #[clap(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();
    println!("{:?}", args);

    match read_file(args.filepath.as_str()) {
        Err(e) => if args.debug {println!("{:?}", e)} else {println!("{}", e)},
        Ok(m) => println!("{}", m),
    }
}

fn read_file(file: &str) -> Result<&str,CompilerError> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|el| {el.expect("There was an unexpected issue.")}).fold(String::from(""), |mut acc,el| {
        acc.push('\n');
        acc.push_str(el.as_str());
        acc
    });
    println!("{}", lines);
    return Ok("passed");
}
