use std::collections::VecDeque;
use crate::lib::tokenizer::*;
use crate::lib::errors::*;
use crate::lib::instructions::*;

pub type Buffer = f32;

pub trait Instruction {
    fn evaluate(&self) -> Result<Buffer, CreateError>;
    fn write(&mut self, value: Buffer) -> CreateResult;
    fn is_full(&self) -> Result<bool, CreateError>;
}

type Buffers = VecDeque<Buffer>;
type Writers = VecDeque<Box<dyn Instruction>>;

fn write(buffers: &mut Buffers, writers: &mut Writers, buf: Buffer) -> CreateResult {
    if writers.is_empty() {
        buffers.push_front(buf);
        CreateResult::Ok()
    } else {
        writers[0].write(buf);
        if match writers[0].is_full() {
            Ok(v) => v,
            Err(e) => return CreateResult::Err(e),
        } {
            let curwriter = writers.pop_front().unwrap();
            write(buffers, writers, match curwriter.evaluate() {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            });
        }
        CreateResult::Ok()
    }
}

pub fn interpret_program(data: Vec<Token>) -> CreateResult {
    let mut program = data.clone();
    program.reverse();
    let mut writers: Writers = Writers::new();
    let mut buffers: Buffers = Buffers::new();
    while let Some(token) = program.pop() {
        use Token::*;
        use Command::*;
        match token {
            CMD(cmd) => {
                writers.push_front(match cmd {
                    ADD => Box::new(Add::new()),
                    SUB => Box::new(Sub::new()),
                    MUL => Box::new(Mul::new()),
                    DIV => Box::new(Div::new()),
                    MOD => Box::new(Mod::new()),
                })
            },
            NUM(num) => {
                write(&mut buffers, &mut writers, num);
            },
            SPC(spc) => {
                match spc {
                    Special::BUF() => {
                        let buf = *match buffers.get(0) {
                            Some(v) => v,
                            None => return CreateResult::Err(CreateError { code: 4, message: "Tried to get value from nonexistent buffer 0".to_string() }),
                        };
                        write(&mut buffers, &mut writers, buf);
                    },
                    Special::IBF(i) => {
                        let buf = *match buffers.get(i) {
                            Some(v) => v,
                            None => return CreateResult::Err(CreateError { code: 4, message: format!("Tried to get value from nonexistent buffer {}", i)})
                        };
                        write(&mut buffers, &mut writers, buf);
                    }
                }
            }
        };
    }
    println!("{:?}", buffers[0]);
    CreateResult::Ok()
}