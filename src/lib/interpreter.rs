#![allow(non_camel_case_types)]
use std::collections::{VecDeque, HashMap};
use std::rc::Rc;
use std::cell::RefCell;
use crate::lib::tokenizer::*;
use crate::lib::errors::*;
use crate::lib::instructions::*;
use crate::lib::controllers::*;

pub type Buffer = f32;

#[derive(Clone, Debug)]
pub struct MutableBuffer(Vec<CreateDirective>);

impl MutableBuffer {
    pub fn new() -> Self {
        MutableBuffer(Vec::new())
    }

    pub fn evaluate(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        while let Some(_) = self.last() {
            match run_directive_tokenless(&mut self.0, environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
        }
        CreateResult::Ok()
    }

    pub fn evaluate_clone(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        let mut cloned_directives = self.iter().map(|x| x.clone()).collect::<Vec<CreateDirective>>();
        while let Some(_) = cloned_directives.last() {
            match run_directive_tokenless(&mut cloned_directives, environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
        }
        CreateResult::Ok()
    }
}

impl std::ops::Deref for MutableBuffer {
    type Target = Vec<CreateDirective>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MutableBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub enum CreateDirective {
    READ_BUF(),
    READ_IBF(usize),
    READ_NBF(String),
    WRITE_INS(Rc<RefCell<dyn Instruction>>),
    WRITE_BUF(Buffer),
    WRITE_NBF(String),
    WRITE_GNB(String),
    CONTROL(Rc<RefCell<dyn Controller>>),
    REMOVE_BUF(),
}

impl Clone for CreateDirective {
    fn clone(&self) -> Self {
        use CreateDirective::*;
        match self {
            READ_BUF() => READ_BUF(),
            READ_IBF(i) => READ_IBF(i.clone()),
            READ_NBF(n) => READ_NBF(n.clone()),
            WRITE_INS(i) => WRITE_INS(i.borrow().clone_ins()),
            WRITE_BUF(b) => WRITE_BUF(b.clone()),
            WRITE_NBF(n) => WRITE_NBF(n.clone()),
            WRITE_GNB(n) => WRITE_GNB(n.clone()),
            CONTROL(c) => CONTROL(c.borrow().clone_cfl()),
            REMOVE_BUF() => REMOVE_BUF(),
        }
    }
}

impl std::fmt::Debug for CreateDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CreateDirective::*;
        let string = match self {
            READ_BUF() => "READ_BUF".to_string(),
            READ_IBF(i) => format!("READ_IBF({})", i),
            READ_NBF(n) => format!("READ_NBF({})", n),
            WRITE_INS(_) => "WRITE_INS(...)".to_string(),
            WRITE_BUF(b) => format!("WRITE_BUF({})", b),
            WRITE_NBF(n) => format!("WRITE_NBF({})", n),
            WRITE_GNB(n) => format!("WRITE_GNB({})", n),
            CONTROL(_) => "CONTROL(...)".to_string(),
            REMOVE_BUF() => "REMOVE_BUF".to_string(),
        };
        write!(f, "{}", string)
    }
}

#[derive(Clone, Debug)]
pub enum CreateAny {
    BUF(Buffer)
}

pub trait Instruction {
    fn evaluate(&mut self, lossy: bool) -> Result<Buffer, CreateError>;
    fn write_buffer(&mut self, _value: Buffer) -> CreateResult {
        CreateResult::Err(CreateError { code: 7, message: "Tried to write a buffer to an incompatible instruction.".to_string() })
    }
    fn is_full(&self) -> Result<bool, CreateError>;
    fn capacity(&self) -> Result<usize, CreateError>; 
    fn clone_ins(&self) -> Rc<RefCell<dyn Instruction>>;
}

pub trait Controller {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult;
    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>>;
}

pub trait Scoping {
    fn get_buf(&self, key: &String) -> Result<&Buffer, CreateError>;
    fn insert(&mut self, key: String, value: CreateAny) -> Option<CreateAny>;
    fn insert_globally(&mut self, key: String, value: CreateAny) -> Option<CreateAny> {
        self.insert(key, value)
    }
    fn insert_locally(&mut self, key: String, value: CreateAny) -> Option<CreateAny> {
        self.insert(key, value)
    }
    fn contains_key(&self, key: &String) -> bool;
    fn scope_type(&self) -> &str {"unknown"}
}

pub type Buffers = VecDeque<Buffer>;
pub type Writers = VecDeque<Rc<RefCell<dyn Instruction>>>;
pub type PrimitiveScope = HashMap<String, CreateAny>;

pub struct Environment<'a> {
    pub buffers: &'a mut Buffers,
    pub writers: &'a mut Writers,
    pub scope: &'a mut dyn Scoping,
}

pub struct Scope<'a> {
    parent: Box<&'a mut dyn Scoping>,
    scope: PrimitiveScope,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Box<&'a mut dyn Scoping>) -> Self {
        Scope { parent, scope: PrimitiveScope::new() }
    }
}

impl<'a> Scoping for Scope<'a> {
    fn get_buf(&self, key: &String) -> Result<&Buffer, CreateError> {
        match self.scope.get_buf(key) {
            Ok(v) => Ok(v),
            Err(_) => self.parent.get_buf(key),
        }
    }
    fn insert(&mut self, key: String, value: CreateAny) -> Option<CreateAny> {
        if self.parent.contains_key(&key) {
            self.parent.insert(key, value)
        } else {
            self.scope.insert(key, value)
        }
    }
    fn insert_globally(&mut self, key: String, value: CreateAny) -> Option<CreateAny> {
        self.parent.insert_globally(key, value)
    }
    fn insert_locally(&mut self, key: String, value: CreateAny) -> Option<CreateAny> {
        self.scope.insert(key, value)
    }
    fn contains_key(&self, key: &String) -> bool {
        self.scope.contains_key(key) || self.parent.contains_key(key)
    }
    fn scope_type(&self) -> &str {"regular"}
}

impl Scoping for PrimitiveScope {
    fn get_buf(&self, key: &String) -> Result<&Buffer, CreateError> {
        match self.get(key) {
            Some(CreateAny::BUF(v)) => Ok(v),
            _ => Err(CreateError { code: 6, message: format!("Could not read buffer from named buffer {}", key) }),
        }
    }
    fn insert(&mut self, key: String, value: CreateAny) -> Option<CreateAny> {
        self.insert(key, value)
    }
    fn contains_key(&self, key: &String) -> bool {
        self.contains_key(key)
    }
    fn scope_type(&self) -> &str {"primitive"}
}

fn write(buffers: &mut Buffers, writers: &mut Writers, buf: Buffer, lossy: bool) -> CreateResult {
    if writers.is_empty() {
        buffers.push_front(buf);
        CreateResult::Ok()
    } else {
        writers[0].borrow_mut().write_buffer(buf);
        if match writers[0].borrow_mut().is_full() {
            Ok(v) => v,
            Err(e) => return CreateResult::Err(e),
        } {
            let curwriter = writers.pop_front().unwrap();
            write(buffers, writers, match curwriter.borrow_mut().evaluate(lossy) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            }, lossy);
        }
        CreateResult::Ok()
    }
}

pub fn write_token(tokens: &mut Vec<Token>, environment: &mut Environment) -> CreateResult {
    match read_token(tokens) {
        Ok(d) => run_directive(d, tokens, environment, true),
        Err(e) => CreateResult::Err(e),
    }
}

pub fn run_directive(directive: CreateDirective, tokens: &mut Vec<Token>, environment: &mut Environment, lossy: bool) -> CreateResult {
    use CreateDirective::*;
    match directive {
        READ_BUF() => write(environment.buffers, environment.writers, match environment.buffers.get(0) {
            Some(v) => *v,
            None => return CreateResult::Err(CreateError { code: 4, message: "Could not read buffer at index 0".to_string() }),
        }, lossy),
        READ_IBF(i) => write(environment.buffers, environment.writers, match environment.buffers.get(i) {
            Some(v) => *v,
            None => return CreateResult::Err(CreateError { code: 4, message: format!("Could not read buffer at index {}", i) }),
        }, lossy),
        READ_NBF(n) => write(environment.buffers, environment.writers, match environment.scope.get_buf(&n) {
            Ok(v) => *v,
            Err(e) => return CreateResult::Err(e),
        }, lossy),
        WRITE_BUF(b) => write(environment.buffers, environment.writers, b, lossy),
        WRITE_INS(i) => {
            environment.writers.push_front(i);
            CreateResult::Ok()
        },
        WRITE_NBF(n) => {
            let mut mutbuffer = match read_mutable_buffer(tokens, Some(1)) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert(n, match environment.buffers.get(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_GNB(n) => {
            let mut mutbuffer = match read_mutable_buffer(tokens, Some(1)) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_globally(n, match environment.buffers.get(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        CONTROL(c) => {
            c.borrow_mut().run(environment, lossy)
        },
        REMOVE_BUF() => {
            environment.buffers.pop_front();
            CreateResult::Ok()
        },
    }
}

pub fn run_directive_tokenless(directives: &mut Vec<CreateDirective>, environment: &mut Environment, lossy: bool) -> CreateResult {
    use CreateDirective::*;
    let directive = match directives.pop() {
        Some(v) => v,
        None => return CreateResult::Err(CreateError { code: usize::MAX, message: "Something went wrong.".to_string() })
    };
    match directive {
        READ_BUF() => write(environment.buffers, environment.writers, match environment.buffers.get(0) {
            Some(v) => *v,
            None => return CreateResult::Err(CreateError { code: 4, message: "Could not read buffer at index 0".to_string() }),
        }, lossy),
        READ_IBF(i) => write(environment.buffers, environment.writers, match environment.buffers.get(i) {
            Some(v) => *v,
            None => return CreateResult::Err(CreateError { code: 4, message: format!("Could not read buffer at index {}", i) }),
        }, lossy),
        READ_NBF(n) => write(environment.buffers, environment.writers, match environment.scope.get_buf(&n) {
            Ok(v) => *v,
            Err(e) => return CreateResult::Err(e),
        }, lossy),
        WRITE_BUF(b) => {
            write(environment.buffers, environment.writers, b, lossy)
        },
        WRITE_INS(i) => {
            environment.writers.push_front(i);
            CreateResult::Ok()
        },
        WRITE_NBF(n) => {
            let mut mutbuffer = match read_mutable_buffer_tokenless(directives) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert(n, match environment.buffers.get(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_GNB(n) => {
            let mut mutbuffer = match read_mutable_buffer_tokenless(directives) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_globally(n, match environment.buffers.get(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        CONTROL(c) => {
            c.borrow_mut().run(environment, lossy)
        },
        REMOVE_BUF() => {
            environment.buffers.pop_front();
            CreateResult::Ok()
        }
    }
}

pub fn read_token(tokens: &mut Vec<Token>) -> Result<CreateDirective, CreateError> {
    use Token::*;
    use Command::*;
    use Special::*;
    use ControlFlow::*;
    let token = match tokens.pop() {
        Some(t) => t,
        None => return Err(CreateError { code: usize::MAX, message: "internal error".to_string() }),
    };
    match token {
        CMD(cmd) => {
            Ok(CreateDirective::WRITE_INS(match cmd {
                ADD => Rc::new(RefCell::new(BinaryOp::new(|l,r| {l+r}))),
                SUB => Rc::new(RefCell::new(BinaryOp::new(|l,r| {l-r}))),
                MUL => Rc::new(RefCell::new(BinaryOp::new(|l,r| {l*r}))),
                DIV => Rc::new(RefCell::new(BinaryOp::new(|l,r| {l/r}))),
                MOD => Rc::new(RefCell::new(BinaryOp::new(|l,r| {l%r}))),
                POW => Rc::new(RefCell::new(BinaryOp::new(|l,r| {l.powf(r)}))),
                SIN => Rc::new(RefCell::new(UnaryOp::new(|v| {v.sin()}))),
                COS => Rc::new(RefCell::new(UnaryOp::new(|v| {v.cos()}))),
                TAN => Rc::new(RefCell::new(UnaryOp::new(|v| {v.tan()}))),
                ASN => Rc::new(RefCell::new(UnaryOp::new(|v| {v.asin()}))),
                ACS => Rc::new(RefCell::new(UnaryOp::new(|v| {v.acos()}))),
                ATN => Rc::new(RefCell::new(UnaryOp::new(|v| {v.atan()}))),
                SQT => Rc::new(RefCell::new(UnaryOp::new(|v| {v.sqrt()}))),
                CBT => Rc::new(RefCell::new(UnaryOp::new(|v| {v.cbrt()}))),
                EQU => Rc::new(RefCell::new(BinaryOp::new(|l,r| {
                    if l == r {1.} else {0.}
                }))),
                NOT => Rc::new(RefCell::new(UnaryOp::new(|l| {
                    if l != 0. {0.} else {1.}
                }))),
                GTH => Rc::new(RefCell::new(BinaryOp::new(|l,r| {
                    if l > r {1.} else {0.}
                }))),
                LTH => Rc::new(RefCell::new(BinaryOp::new(|l,r| {
                    if l < r {1.} else {0.}
                }))),
                ORR => Rc::new(RefCell::new(BinaryOp::new(|l,r| {
                    if l == 1. || r == 1. {1.} else {0.}
                }))),
                AND => Rc::new(RefCell::new(BinaryOp::new(|l,r| {
                    if l == 1. && r == 1. {1.} else {0.}
                }))),
                PNT => Rc::new(RefCell::new(UnaryOp::new(|v| {print!("{}", v); v}))),
                PTC => Rc::new(RefCell::new(UnaryOp::new(|v| {
                    let tv = v.trunc();
                    print!("{}",(tv as u8) as char);
                    tv
                }))),
            }))
        },
        NUM(num) => {
            Ok(CreateDirective::WRITE_BUF(num))
        },
        SPC(spc) => {
            match spc {
                RMB() => Ok(CreateDirective::REMOVE_BUF()),
                BUF() => Ok(CreateDirective::READ_BUF()),
                IBF(i) => Ok(CreateDirective::READ_IBF(i)),
                SNB(n) => Ok(CreateDirective::WRITE_NBF(n)),
                SGB(n) => Ok(CreateDirective::WRITE_GNB(n)),
                GNB(n) => Ok(CreateDirective::READ_NBF(n)),
                OPB() => {
                    let mut scopedbuffers: Vec<MutableBuffer> = Vec::new();
                    let mut scopedtokens: Vec<Token> = Vec::new();
                    while let Some(token) = tokens.pop() {
                        if let SPC(CLB()) = token {break}
                        scopedtokens.push(token);
                    }
                    scopedtokens.reverse();
                    while let Some(_) = scopedtokens.last() {
                        scopedbuffers.push(read_mutable_buffer(&mut scopedtokens, None)?)
                    }
                    scopedbuffers.reverse();
                    let control = Scoped::new(scopedbuffers);
                    Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(control))))
                },
                CLB() => Err(CreateError { code: 3, message: "Unexpected closing bracket.".to_string() }),
            }
        },
        CFL(cfl) => {
            match cfl {
                IFF => {
                    let condition = read_mutable_buffer(tokens, None)?;
                    let ifmutbuffer = read_mutable_buffer(tokens, None)?;
                    if let Some(CFL(ELS)) = tokens.last() {
                        tokens.pop();
                        Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(IfElse::new(condition, ifmutbuffer, read_mutable_buffer(tokens, None)?)))))
                    } else {
                        let control = If::new(condition, ifmutbuffer);
                        Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(control))))
                    }
                },
                FOR => {
                    let mut identifier: Option<String> = None;
                    if let Some(SPC(SNB(i))) = tokens.last() {
                        identifier = Some(i.clone());
                        tokens.pop();
                    }
                    let condition = read_mutable_buffer(tokens, None)?;
                    let control = For::new(condition, identifier, read_mutable_buffer(tokens, None)?);
                    Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(control))))
                },
                WHL => {
                    let control = While::new(read_mutable_buffer(tokens, None)?, read_mutable_buffer(tokens, None)?);
                    Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(control))))
                }
                _ => Err(CreateError { code: 3, message: "Unexpected control flow token found.".to_string() }),
            }
        }
    }
}

pub fn read_value(tokens: &mut Vec<Token>, environment: &mut Environment) -> Result<CreateAny, CreateError> {
    while let Some(_) = tokens.last() {
        write_token(tokens, environment);
        if environment.buffers.len() > 0 {break}
    }
    Ok(CreateAny::BUF(match environment.buffers.get(0) {
        Some(v) => *v,
        None => return Err(CreateError{ code: usize::MAX, message: "There was an issue.".to_string() })
    }))
}

pub fn read_value_tokenless(directives: &mut Vec<CreateDirective>, environment: &mut Environment, lossy: bool) -> Result<CreateAny, CreateError> {
    while let Some(_) = directives.last() {
        match run_directive_tokenless(directives, environment, lossy) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => return Err(e),
        }
        if environment.buffers.len() > 0 {break}
    }
    Ok(CreateAny::BUF(environment.buffers[0]))
}

pub fn read_mutable_buffer(tokens: &mut Vec<Token>, capacity: Option<i32>) -> Result<MutableBuffer, CreateError> {
    let mut mutbuffer = MutableBuffer::new();
    let mut capacity = match capacity {
        Some(v) => vec![v],
        None => Vec::new(),
    };
    'main: while let Some(_) = tokens.last() {
        use CreateDirective::*;
        let current = read_token(tokens)?;
        mutbuffer.push(current);
        match mutbuffer.last().unwrap() {
            READ_BUF()
            | READ_IBF(_)
            | READ_NBF(_)
            | WRITE_BUF(_) => {
                'rec: loop {
                    match capacity.last_mut() { 
                        Some(v) => {
                            *v -= 1;
                            if capacity.last().unwrap() <= &0 {
                                capacity.pop();
                                continue 'rec;
                            }
                            break 'rec;
                        },
                        None => break 'main,
                    };
                }
            },
            WRITE_INS(i) => {
                capacity.push(i.borrow().capacity()? as i32)
            },
            WRITE_GNB(_)
            | WRITE_NBF(_) => {
                capacity.push(1);
            },
            _ => (),
        }
        if capacity.is_empty() {break}
    }
    mutbuffer.reverse();
    Ok(mutbuffer)
}

pub fn read_mutable_buffer_tokenless(directives: &mut Vec<CreateDirective>) -> Result<MutableBuffer, CreateError> {
    let mut mutbuffer = MutableBuffer::new();
    let mut capacity = Vec::new();
    'main: while let Some(directive) = directives.pop() {
        use CreateDirective::*;
        mutbuffer.push(directive);
        match mutbuffer.last().unwrap() {
            READ_BUF()
            | READ_IBF(_)
            | READ_NBF(_)
            | WRITE_BUF(_) => {
                'rec: loop {
                    match capacity.last_mut() {
                        Some(v) => {
                            *v -= 1;
                            if capacity.last().unwrap() <= &0 {
                                capacity.pop();
                                continue 'rec;
                            }
                            break 'rec;
                        },
                        None => break 'main,
                    }
                }
            },
            WRITE_INS(i) => {
                capacity.push(i.borrow().capacity()? as i32);
            },
            WRITE_GNB(_)
            | WRITE_NBF(_) => capacity.push(1),
            _ => (),
        }
        if capacity.is_empty() {break}
    }
    mutbuffer.reverse();
    Ok(mutbuffer)
}

pub fn interpret_program(data: Vec<Token>) -> CreateResult {
    let mut program = data.clone();
    program.reverse();
    let mut writers: Writers = Writers::new();
    let mut buffers: Buffers = Buffers::new();
    let mut scope: PrimitiveScope = PrimitiveScope::new();
    let mut environment: Environment = Environment { 
        writers: &mut writers, 
        buffers: &mut buffers, 
        scope: &mut scope,
    };
    while let Some(_) = program.last() {
        match write_token(&mut program, &mut environment) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => return CreateResult::Err(e),
        }
    }
    CreateResult::Ok()
}