use std::collections::{VecDeque, HashMap};
use crate::lib::tokenizer::*;
use crate::lib::errors::*;
use crate::lib::instructions::*;

pub type Buffer = f32;

#[derive(Clone)]
pub enum CreateAny {
    BUF(Buffer)
}

pub trait Instruction {
    fn evaluate(&mut self) -> Result<Buffer, CreateError>;
    fn write_buffer(&mut self, _value: Buffer) -> CreateResult {
        CreateResult::Err(CreateError { code: 7, message: "Tried to write a buffer to an incompatible instruction.".to_string() })
    }
    fn is_full(&self) -> Result<bool, CreateError>;
}

type Buffers = VecDeque<Buffer>;
type Writers = VecDeque<Box<dyn Instruction>>;
type Scope = HashMap<String, CreateAny>;

fn write(buffers: &mut Buffers, writers: &mut Writers, buf: Buffer) -> CreateResult {
    if writers.is_empty() {
        buffers.push_front(buf);
        CreateResult::Ok()
    } else {
        writers[0].write_buffer(buf);
        if match writers[0].is_full() {
            Ok(v) => v,
            Err(e) => return CreateResult::Err(e),
        } {
            let mut curwriter = writers.pop_front().unwrap();
            write(buffers, writers, match curwriter.evaluate() {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            });
        }
        CreateResult::Ok()
    }
}

pub fn read_token(tokens: &mut Vec<Token>, buffers: &mut Buffers, writers: &mut Writers, named_buffers: &mut HashMap<String, CreateAny>) -> CreateResult {
    use Token::*;
    use Command::*;
    let token = match tokens.pop() {
        Some(t) => t,
        None => return CreateResult::Err(CreateError { code: usize::MAX, message: "internal error".to_string() }),
    };
    match token {
        CMD(cmd) => {
            writers.push_front(match cmd {
                ADD => Box::new(BinaryOp::new(Box::new(|l,r| {l+r}))),
                SUB => Box::new(BinaryOp::new(Box::new(|l,r| {l-r}))),
                MUL => Box::new(BinaryOp::new(Box::new(|l,r| {l*r}))),
                DIV => Box::new(BinaryOp::new(Box::new(|l,r| {l/r}))),
                MOD => Box::new(BinaryOp::new(Box::new(|l,r| {l%r}))),
                POW => Box::new(BinaryOp::new(Box::new(|l,r| {l.powf(r)}))),
                SIN => Box::new(UnaryOp::new(Box::new(|v| {v.sin()}))),
                COS => Box::new(UnaryOp::new(Box::new(|v| {v.cos()}))),
                TAN => Box::new(UnaryOp::new(Box::new(|v| {v.tan()}))),
                ASN => Box::new(UnaryOp::new(Box::new(|v| {v.asin()}))),
                ACS => Box::new(UnaryOp::new(Box::new(|v| {v.acos()}))),
                ATN => Box::new(UnaryOp::new(Box::new(|v| {v.atan()}))),
                SQT => Box::new(UnaryOp::new(Box::new(|v| {v.sqrt()}))),
                CBT => Box::new(UnaryOp::new(Box::new(|v| {v.cbrt()}))),
                EQU => Box::new(BinaryOp::new(Box::new(|l,r| {
                    if l == r {1.} else {0.}
                }))),
                NOT => Box::new(UnaryOp::new(Box::new(|l| {
                    if l != 0. {0.} else {1.}
                }))),
                GTH => Box::new(BinaryOp::new(Box::new(|l,r| {
                    if l > r {1.} else {0.}
                }))),
                LTH => Box::new(BinaryOp::new(Box::new(|l,r| {
                    if l < r {1.} else {0.}
                }))),
                ORR => Box::new(BinaryOp::new(Box::new(|l,r| {
                    if l == 1. || r == 1. {1.} else {0.}
                }))),
                AND => Box::new(BinaryOp::new(Box::new(|l,r| {
                    if l == 1. && r == 1. {1.} else {0.}
                }))),
                PNT => Box::new(UnaryOp::new(Box::new(|v| {print!("{}", v); v}))),
                PTC => Box::new(UnaryOp::new(Box::new(|v| {
                    let tv = v.trunc();
                    print!("{}",(tv as u8) as char);
                    tv
                }))),
            })
        },
        NUM(num) => {
            write(buffers, writers, num);
        },
        SPC(spc) => {
            match spc {
                Special::RMB() => {
                    buffers.pop_front();
                }
                Special::BUF() => {
                    let buf = *match buffers.get(0) {
                        Some(v) => v,
                        None => return CreateResult::Err(CreateError { code: 4, message: "Tried to get value from nonexistent buffer 0".to_string() }),
                    };
                    write(buffers, writers, buf);
                },
                Special::IBF(i) => {
                    let buf = *match buffers.get(i) {
                        Some(v) => v,
                        None => return CreateResult::Err(CreateError { code: 4, message: format!("Tried to get value from nonexistent buffer {}", i)})
                    };
                    write(buffers, writers, buf);
                },
                Special::SNB(n) => {
                    named_buffers.insert(n, read_value(tokens));
                },
                Special::GNB(n) => {
                    let buf = match named_buffers.get(&n) {
                        Some(v) => v,
                        None => return CreateResult::Err(CreateError { code: 7, message: format!("Tried to get value from nonexistent buffer {}", n) }),
                    };
                    match buf {
                        CreateAny::BUF(b) => write(buffers, writers, *b),
                    };
                }
            }
        }
    };
    CreateResult::Ok()
}

pub fn read_value(tokens: &mut Vec<Token>) -> CreateAny {
    let mut writers: Writers = Writers::new();
    let mut buffers: Buffers = Buffers::new();
    let mut named_buffers: Scope = Scope::new();
    while let Some(_) = tokens.last() {
        read_token(tokens, &mut buffers, &mut writers, &mut named_buffers);
        if buffers.len() > 0 {break}
    }
    CreateAny::BUF(buffers[0])
}

pub fn interpret_program(data: Vec<Token>) -> CreateResult {
    let mut program = data.clone();
    program.reverse();
    let mut writers: Writers = Writers::new();
    let mut buffers: Buffers = Buffers::new();
    let mut named_buffers: Scope = Scope::new();
    while let Some(_) = program.last() {
        match read_token(&mut program, &mut buffers, &mut writers, &mut named_buffers) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => return CreateResult::Err(e),
        }
    }
    println!("{:?}", buffers.get(0));
    CreateResult::Ok()
}