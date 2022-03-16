#![allow(non_camel_case_types)]
use std::collections::{VecDeque, HashMap};
use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;
use super::tokenizer::*;
use super::errors::*;
use super::instructions::*;
use super::controllers::*;
use super::functions::*;

pub type Buffer = f32;
pub type Array = Vec<CreateAny>;

impl Into<CreateAny> for Buffer {
    fn into(self) -> CreateAny {
        CreateAny::BUF(self)
    }
}

impl Into<CreateAny> for Array {
    fn into(self) -> CreateAny {
        CreateAny::ARR(self)
    }
}

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

    pub fn evaluate_clone(&self, environment: &mut Environment, lossy: bool) -> CreateResult {
        let mut cloned_directives = self.iter().map(|x| x.clone()).collect::<Vec<CreateDirective>>();
        while let Some(_) = cloned_directives.last() {
            match run_directive_tokenless(&mut cloned_directives, environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
        }
        CreateResult::Ok()
    }

    pub fn eval_return(&mut self, environment: &mut Environment, lossy: bool) -> Result<Option<Box<CreateAny>>, CreateError> {
        let mut exposed_buffer = PartitionedBuffers::new(environment.buffers);
        self.evaluate(&mut Environment { buffers: &mut exposed_buffer, writers: &mut Writers::new(), scope: environment.scope }, lossy);
        match exposed_buffer.get_return() {
            Some(v) => Ok(Some(v)),
            None => Ok(None),
        }
    }

    pub fn eval_clone_return(&self, environment: &mut Environment, lossy: bool) -> Result<Option<Box<CreateAny>>, CreateError> {
        let mut exposed_buffers = PartitionedBuffers::new(environment.buffers);
        self.evaluate_clone(&mut Environment { buffers: &mut exposed_buffers, writers: &mut Writers::new(), scope: environment.scope }, lossy);
        match exposed_buffers.get_return() {
            Some(v) => Ok(Some(v)),
            None => Ok(None),
        }
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
    READ_IAR(String, MutableBuffer),
    READ_LIA(String, Vec<MutableBuffer>),
    READ_NBF(String),
    WRITE_INS(Rc<RefCell<dyn Instruction>>),
    WRITE_BUF(Buffer),
    WRITE_ARR(Vec<MutableBuffer>),
    WRITE_NBF(String),
    WRITE_NAR(String),
    WRITE_NFN(String, Function),
    WRITE_GNB(String),
    WRITE_GNA(String),
    WRITE_LNB(String),
    WRITE_LNA(String),
    CONTROL(Rc<RefCell<dyn Controller>>),
    BREAK(),
    RETURN(),
    REMOVE_BUF(),
}

impl Clone for CreateDirective {
    fn clone(&self) -> Self {
        use CreateDirective::*;
        match self {
            READ_BUF() => READ_BUF(),
            READ_IBF(i) => READ_IBF(i.clone()),
            READ_IAR(n, m) => READ_IAR(n.clone(), m.clone()),
            READ_LIA(n,m) => READ_LIA(n.clone(), m.clone()),
            READ_NBF(n) => READ_NBF(n.clone()),
            WRITE_INS(i) => WRITE_INS(i.borrow().clone_ins()),
            WRITE_BUF(b) => WRITE_BUF(b.clone()),
            WRITE_ARR(a) => WRITE_ARR(a.clone()),
            WRITE_NBF(n) => WRITE_NBF(n.clone()),
            WRITE_NAR(n) => WRITE_NAR(n.clone()),
            WRITE_NFN(n, f) => WRITE_NFN(n.clone(), f.clone()),
            WRITE_GNB(n) => WRITE_GNB(n.clone()),
            WRITE_GNA(n) => WRITE_GNA(n.clone()),
            WRITE_LNB(n) => WRITE_LNB(n.clone()),
            WRITE_LNA(n) => WRITE_LNA(n.clone()),
            CONTROL(c) => CONTROL(c.borrow().clone_cfl()),
            BREAK() => BREAK(),
            RETURN() => RETURN(),
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
            READ_IAR(n, m) => format!("READ_IAR({}, {:?})", n, m),
            READ_LIA(n, m) => format!("READ_LIA({}, {:?})", n, m),
            READ_NBF(n) => format!("READ_NBF({})", n),
            WRITE_INS(_) => "WRITE_INS(...)".to_string(),
            WRITE_BUF(b) => format!("WRITE_BUF({})", b),
            WRITE_ARR(a) => format!("WRITE_ARR({:?})", a),
            WRITE_NBF(n) => format!("WRITE_NBF({})", n),
            WRITE_NAR(n) => format!("WRITE_NAR({})", n),
            WRITE_NFN(n, f) => format!("WRITE_NFN({}, {:?})", n, f),
            WRITE_LNB(n) => format!("WRITE_LNB({})", n),
            WRITE_LNA(n) => format!("WRITE_LNA({})", n),
            WRITE_GNB(n) => format!("WRITE_GNB({})", n),
            WRITE_GNA(n) => format!("WRITE_GNA({})", n),
            CONTROL(_) => "CONTROL(...)".to_string(),
            BREAK() => "BREAK".to_string(),
            RETURN() => "RETURN".to_string(),
            REMOVE_BUF() => "REMOVE_BUF".to_string(),
        };
        write!(f, "{}", string)
    }
}

#[derive(Clone, Debug)]
pub enum CreateType {
    BUF,
    ARR,
    FUN,
    NUL,
}

impl CreateType {
    pub fn matches(&self, val: &CreateAny) -> bool {
        match (self, val) {
            (CreateType::BUF, CreateAny::BUF(_)) => true,
            (CreateType::ARR, CreateAny::ARR(_)) => true,
            (CreateType::FUN, CreateAny::FUN(_)) => true,
            (CreateType::NUL, CreateAny::NUL()) => true,
            _ => false
        }
    }
}

#[derive(Clone, Debug)]
pub enum CreateAny {
    BUF(Buffer),
    ARR(Array),
    FUN(Function),
    NUL(),
}

pub trait Instruction {
    fn evaluate(&mut self, environment: &mut Environment, lossy: bool) -> Result<CreateAny, CreateError>;
    fn write_buffer(&mut self, _value: CreateAny) -> CreateResult {
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
    fn get(&self, key: &String) -> Result<&CreateAny, CreateError>;
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

pub type Buffers = VecDeque<CreateAny>;
pub type Writers = VecDeque<Rc<RefCell<dyn Instruction>>>;
pub type PrimitiveScope = HashMap<String, CreateAny>;

pub struct PartitionedBuffers<'a> {
    prev: &'a mut dyn Buffering,
    new: Box<dyn Buffering>,
}

#[allow(dead_code)]
impl<'a> PartitionedBuffers<'a> {
    pub fn new(prev: &'a mut dyn Buffering) -> Self {
        PartitionedBuffers { prev, new: Box::new(Buffers::new()) }
    }

    pub fn get_return(self) -> Option<Box<CreateAny>> {
        self.new.get(0)
    }

    pub fn get_return_buf(self) -> Option<Box<Buffer>> {
        self.new.get_buf(0)
    }

    pub fn get_return_arr(self) -> Option<Box<Array>> {
        self.new.get_arr(0)
    }

    pub fn trunc(self) -> &'a mut dyn Buffering {
        self.prev
    }

    pub fn combine(mut self) -> &'a mut dyn Buffering {
        while let Some(v) = self.new.pop() {
            self.prev.push(v);
        }
        self.prev
    }
}

pub trait Buffering {
    fn get(&self, index: usize) -> Option<Box<CreateAny>>;
    fn get_buf(&self, index: usize) -> Option<Box<Buffer>>;
    fn get_arr(&self, index: usize) -> Option<Box<Array>>;
    fn push(&mut self, value: CreateAny);
    fn push_buf(&mut self, value: Buffer);
    fn push_arr(&mut self, value: Array);
    fn pop(&mut self) -> Option<CreateAny>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl Buffering for Buffers {
    fn get(&self, index: usize) -> Option<Box<CreateAny>> {
        match self.get(index) {
            Some(b) => Some(Box::new(b.clone())),
            None => None,
        }
    }

    fn get_buf(&self, index: usize) -> Option<Box<Buffer>> {
        match self.get(index) {
            Some(CreateAny::BUF(b)) => Some(Box::new(b.clone())),
            _ => None,
        }
    }

    fn get_arr(&self, index: usize) -> Option<Box<Array>> {
        match self.get(index) {
            Some(CreateAny::ARR(a)) => Some(Box::new(a.clone())),
            _ => None,
        }
    }

    fn push(&mut self, value: CreateAny) {
        self.push_front(value);
    }

    fn push_buf(&mut self, value: Buffer) {
        self.push_front(CreateAny::BUF(value));
    }

    fn push_arr(&mut self, value: Array) {
        self.push_front(CreateAny::ARR(value));
    }

    fn pop(&mut self) -> Option<CreateAny> {
        self.pop_front()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'a> Buffering for PartitionedBuffers<'a> {
    fn get(&self, index: usize) -> Option<Box<CreateAny>> {
        match self.new.get(index) {
            Some(v) => Some(v),
            None => self.prev.get(index - self.new.len()),
        }
    }

    fn get_buf(&self, index: usize) -> Option<Box<Buffer>> {
        match self.new.get_buf(index) {
            Some(v) => Some(v),
            None => self.prev.get_buf(index - self.new.len()),
        }
    }

    fn get_arr(&self, index: usize) -> Option<Box<Array>> {
        match self.new.get_arr(index) {
            Some(v) => Some(v),
            None => self.prev.get_arr(index - self.new.len()),
        }
    }

    fn push(&mut self, value: CreateAny) {
        self.new.push(value);
    }

    fn push_buf(&mut self, value: Buffer) {
        self.new.push_buf(value);
    }

    fn push_arr(&mut self, value: Array) {
        self.new.push_arr(value);
    }

    fn pop(&mut self) -> Option<CreateAny> {
        if self.new.is_empty() {
            None
        } else {
            self.new.pop()
        }
    }

    fn len(&self) -> usize {
        self.prev.len() + self.new.len()
    }

    fn is_empty(&self) -> bool {
        self.new.is_empty() && self.prev.is_empty()
    }
}

pub struct Environment<'a> {
    pub buffers: &'a mut dyn Buffering,
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
    fn get(&self, key: &String) -> Result<&CreateAny, CreateError> {
        match self.scope.get(key) {
            Some(v) => Ok(v),
            None => self.parent.get(key),
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
    fn get(&self, key: &String) -> Result<&CreateAny, CreateError> {
        match self.get(key) {
            Some(v) => Ok(v),
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

pub fn write(environment: &mut Environment, val: CreateAny, lossy: bool) -> CreateResult {
    if environment.writers.is_empty() {
        environment.buffers.push(val);
        CreateResult::Ok()
    } else {
        environment.writers[0].borrow_mut().write_buffer(val);
        if match environment.writers[0].borrow_mut().is_full() {
            Ok(v) => v,
            Err(e) => return CreateResult::Err(e),
        } {
            let curwriter = environment.writers.pop_front().unwrap();
            let val = match curwriter.borrow_mut().evaluate(environment, lossy) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            write(environment, val, lossy);
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
        READ_BUF() => {
            let buf = match environment.buffers.get_buf(0) {
                Some(v) => *v,
                None => return CreateResult::Err(CreateError { code: 4, message: "Could not read buffer at index 0".to_string() }),
            };
            write(environment, buf.into(), lossy)
        },
        READ_IBF(i) => { 
            let buf = match environment.buffers.get_buf(i) {
                Some(v) => *v,
                None => return CreateResult::Err(CreateError { code: 4, message: format!("Could not read buffer at index {}", i) }),
            };
            write(environment, buf.into(), lossy)
        },
        READ_IAR(n, mut m) => {
            let index = match m.eval_return(environment, lossy) {
                Ok(Some(v)) => match *v {
                    CreateAny::BUF(b) => b,
                    _ => return CreateResult::Err(CreateError { code: 3, message: "Index of array did not return buffer, and none was found.".to_string() }),
                },
                Err(e) => return CreateResult::Err(e),
                _ => return CreateResult::Err(CreateError { code: 3, message: "Index of array did not return buffer, and none was found.".to_string() }),
            };
            let arr = match environment.scope.get(&n) {
                Ok(CreateAny::ARR(a)) => a,
                Ok(_) => return CreateResult::Err(CreateError { code: 3, message: format!("Identifier {} was not an array as expected.", n) }),
                Err(e) => return CreateResult::Err(e),
            };
            let val = match arr.get(index as usize) {
                Some(v) => v,
                None => return CreateResult::Err(CreateError { code: 3, message: format!("Value at index {} in array {} was outside of the array", index, n) })
            }.clone();
            write(environment, val, lossy)
        },
        READ_LIA(n, mut m) => {
            let mut arr = match environment.scope.get(&n) {
                Ok(v) => match v {
                    CreateAny::ARR(a) => a.clone(),
                    _ => return CreateResult::Err(CreateError { code: 3, message: format!("Identifier {} was not an array as expected", n) }),
                },
                Err(e) => return CreateResult::Err(e),
            };
            let mut val = None;
            while let Some(mut mutbuffer) = m.pop() {
                let index = match mutbuffer.eval_return(environment, lossy) {
                    Ok(Some(v)) => match *v {
                        CreateAny::BUF(b) => b,
                        _ => return CreateResult::Err(CreateError { code: 3, message: "Index in long array index did not return buffer".to_string() })
                    },
                    Ok(None) => return CreateResult::Err(CreateError { code: 3, message: "Index in long array index cannot be null".to_string() }),
                    Err(e) => return CreateResult::Err(e),
                };
                if let None = m.last() {
                    val = arr.get(index as usize);
                    break;
                }
                arr = match arr.get(index as usize) {
                    Some(CreateAny::ARR(a)) => a.clone(),
                    Some(_) => return CreateResult::Err(CreateError { code: 3, message: "Non-final index in long array index did not resolve to array".to_string() }),
                    None => return CreateResult::Err(CreateError { code: 3, message: "Index in long array index could not be resolved".to_string() }),
                };
            }
            write(environment, match val {
                Some(v) => v.clone(),
                None => return CreateResult::Err(CreateError { code: 3, message: "Long array index cannot return null value".to_string() })
            }, lossy)
        },
        READ_NBF(n) => write(environment, match environment.scope.get(&n) {
            Ok(v) => v.clone(),
            Err(e) => return CreateResult::Err(e),
        }, lossy),
        WRITE_BUF(b) => write(environment, b.into(), lossy),
        WRITE_ARR(m) => {
            let mut arr = Array::new();
            for mut mutbuffer in m {
                arr.push(match mutbuffer.eval_return(environment, lossy) {
                    Ok(Some(v)) => *v,
                    Ok(None) => return CreateResult::Err(CreateError { code: 3, message: "Arrays cannot contain null values.".to_string() }),
                    Err(e) => return CreateResult::Err(e),
                });
            }
            write(environment, arr.into(), lossy)
        }
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
            environment.scope.insert(n, match environment.buffers.get_buf(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_NAR(n) => {
            let mut mutbuffer = match read_mutable_buffer(tokens, Some(1)) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert(n, match environment.buffers.get_arr(0) {
                Some(a) => CreateAny::ARR(*a),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_NFN(n, f) => {
            environment.scope.insert(n, CreateAny::FUN(f));
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
            environment.scope.insert_globally(n, match environment.buffers.get_buf(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_GNA(n) => {
            let mut mutbuffer = match read_mutable_buffer(tokens, Some(1)) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_globally(n, match environment.buffers.get_arr(0) {
                Some(a) => CreateAny::ARR(*a),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_LNB(n) => {
            let mut mutbuffer = match read_mutable_buffer(tokens, Some(1)) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_locally(n, match environment.buffers.get_buf(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_LNA(n) => {
            let mut mutbuffer = match read_mutable_buffer(tokens, Some(1)) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_locally(n, match environment.buffers.get_arr(0) {
                Some(a) => CreateAny::ARR(*a),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        CONTROL(c) => {
            c.borrow_mut().run(environment, lossy)
        },
        BREAK() => CreateResult::Err(CreateError { code: 11, message: "Found misplaced break statement".to_string() }),
        RETURN() => CreateResult::Err(CreateError { code: 12, message: "Found misplaced return statement".to_string() }),
        REMOVE_BUF() => {
            environment.buffers.pop();
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
        READ_BUF() => {
            let buf = match environment.buffers.get_buf(0) {
                Some(v) => *v,
                None => return CreateResult::Err(CreateError { code: 4, message: "Could not read buffer at index 0".to_string() }),
            };
            write(environment, buf.into(), lossy)
        },
        READ_IBF(i) => { 
            let buf = match environment.buffers.get_buf(i) {
                Some(v) => *v,
                None => return CreateResult::Err(CreateError { code: 4, message: format!("Could not read buffer at index {}", i) }),
            };
            write(environment, buf.into(), lossy)
        },
        READ_NBF(n) => write(environment, match environment.scope.get(&n) {
            Ok(v) => v.clone(),
            Err(e) => return CreateResult::Err(e),
        }, lossy),
        READ_IAR(n, mut m) => {
            let index = match m.eval_return(environment, lossy) {
                Ok(Some(v)) => match *v {
                    CreateAny::BUF(b) => b,
                    _ => return CreateResult::Err(CreateError { code: 3, message: "Index of array did not return buffer, and none was found.".to_string() }),
                },
                Err(e) => return CreateResult::Err(e),
                _ => return CreateResult::Err(CreateError { code: 3, message: "Index of array did not return buffer, and none was found.".to_string() }),
            };
            let arr = match environment.scope.get(&n) {
                Ok(CreateAny::ARR(a)) => a,
                Ok(_) => return CreateResult::Err(CreateError { code: 3, message: format!("Identifier {} was not an array as expected.", n) }),
                Err(e) => return CreateResult::Err(e),
            };
            let val = match arr.get(index as usize) {
                Some(v) => v,
                None => return CreateResult::Err(CreateError { code: 3, message: format!("Value at index {} in array {} was outside of the array", index, n) })
            }.clone();
            environment.buffers.pop();
            write(environment, val, lossy)
        },
        READ_LIA(n, mut m) => {
            let mut arr = match environment.scope.get(&n) {
                Ok(v) => match v {
                    CreateAny::ARR(a) => a.clone(),
                    _ => return CreateResult::Err(CreateError { code: 3, message: format!("Identifier {} was not an array as expected", n) }),
                },
                Err(e) => return CreateResult::Err(e),
            };
            let mut val = None;
            while let Some(mut mutbuffer) = m.pop() {
                let index = match mutbuffer.eval_return(environment, lossy) {
                    Ok(Some(v)) => match *v {
                        CreateAny::BUF(b) => b,
                        _ => return CreateResult::Err(CreateError { code: 3, message: "Index in long array index did not return buffer".to_string() })
                    },
                    Ok(None) => return CreateResult::Err(CreateError { code: 3, message: "Index in long array index cannot be null".to_string() }),
                    Err(e) => return CreateResult::Err(e),
                };
                if let None = m.last() {
                    val = arr.get(index as usize);
                    break;
                }
                arr = match arr.get(index as usize) {
                    Some(CreateAny::ARR(a)) => a.clone(),
                    Some(_) => return CreateResult::Err(CreateError { code: 3, message: "Non-final index in long array index did not resolve to array".to_string() }),
                    None => return CreateResult::Err(CreateError { code: 3, message: "Index in long array index could not be resolved".to_string() }),
                };
            }
            write(environment, match val {
                Some(v) => v.clone(),
                None => return CreateResult::Err(CreateError { code: 3, message: "Long array index cannot return null value".to_string() })
            }, lossy)
        },
        WRITE_BUF(b) => {
            write(environment, b.into(), lossy)
        },
        WRITE_ARR(m) => {
            let mut arr = Array::new();
            for mut mutbuffer in m {
                arr.push(match mutbuffer.eval_return(environment, lossy) {
                    Ok(Some(v)) => *v,
                    Ok(None) => return CreateResult::Err(CreateError { code: 3, message: "Arrays cannot contain null values.".to_string() }),
                    Err(e) => return CreateResult::Err(e),
                });
            }
            write(environment, arr.into(), lossy)
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
            environment.scope.insert(n, match environment.buffers.get_buf(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_NAR(n) => {
            let mut mutbuffer = match read_mutable_buffer_tokenless(directives) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert(n, match environment.buffers.get_arr(0) {
                Some(a) => CreateAny::ARR(*a),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_NFN(n, f) => {
            environment.scope.insert(n, CreateAny::FUN(f));
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
            environment.scope.insert_globally(n, match environment.buffers.get_buf(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_GNA(n) => {
            let mut mutbuffer = match read_mutable_buffer_tokenless(directives) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_globally(n, match environment.buffers.get_arr(0) {
                Some(a) => CreateAny::ARR(*a),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_LNB(n) => {
            let mut mutbuffer = match read_mutable_buffer_tokenless(directives) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_locally(n, match environment.buffers.get_buf(0) {
                Some(b) => CreateAny::BUF(*b),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        WRITE_LNA(n) => {
            let mut mutbuffer = match read_mutable_buffer_tokenless(directives) {
                Ok(v) => v,
                Err(e) => return CreateResult::Err(e),
            };
            match mutbuffer.evaluate(environment, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
            environment.scope.insert_locally(n, match environment.buffers.get_arr(0) {
                Some(a) => CreateAny::ARR(*a),
                None => return CreateResult::Err(CreateError { code: 3, message: "Named buffer was attempted to be set to null.".to_string() })
            });
            CreateResult::Ok()
        },
        CONTROL(c) => {
            c.borrow_mut().run(environment, lossy)
        },
        BREAK() => CreateResult::Err(CreateError { code: 11, message: "Found misplaced break statement".to_string() }),
        RETURN() => CreateResult::Err(CreateError { code: 12, message: "Found misplaced return statement".to_string() }),
        REMOVE_BUF() => {
            environment.buffers.pop();
            CreateResult::Ok()
        },
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
                SNA(n) => Ok(CreateDirective::WRITE_NAR(n)),
                SGB(n) => Ok(CreateDirective::WRITE_GNB(n)),
                SGA(n) => Ok(CreateDirective::WRITE_GNA(n)),
                SLA(n) => Ok(CreateDirective::WRITE_LNA(n)),
                SLB(n) => Ok(CreateDirective::WRITE_LNB(n)),
                GNB(n) => Ok(CreateDirective::READ_NBF(n)),
                FUN(n) => {
                    let mut params: Vec<(CreateType, String)> = Vec::new();
                    if let Some(SPC(OPR())) = tokens.last() {tokens.pop();}
                    else {return Err(CreateError { code: 2, message: "Function was attempted to be defined without argument set.".to_string() })}
                    while let Some(token) = tokens.pop() {
                        match token {
                            SPC(CLR()) => break,
                            SPC(SNB(n)) => params.push((CreateType::BUF, n)),
                            SPC(SNA(n)) => params.push((CreateType::ARR, n)),
                            SPC(FUN(n)) => params.push((CreateType::FUN, n)),
                            _ => return Err(CreateError { code: 3, message: "Function arguments must be a form of setter".to_string() }),
                        }
                    }
                    let fun = Function::new(params, read_mutable_buffer(tokens, None)?);
                    Ok(CreateDirective::WRITE_NFN(n,fun))
                },
                FNC(n) => {
                    let mut parambuffers: Vec<MutableBuffer> = Vec::new();
                    while let Some(token) = tokens.last() {
                        if let SPC(CLR()) = token {tokens.pop(); break}
                        parambuffers.push(read_mutable_buffer(tokens, None)?)
                    }
                    parambuffers.reverse();
                    let fnc = FunctionCall::new(n.clone(), parambuffers);
                    Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(fnc))))
                },
                OPB() => {
                    let mut scopedbuffers: Vec<MutableBuffer> = Vec::new();
                    while let Some(token) = tokens.last() {
                        if let SPC(CLB()) = token {tokens.pop(); break}
                        scopedbuffers.push(read_mutable_buffer(tokens, None)?)
                    }
                    scopedbuffers.reverse();
                    let control = Scoped::new(scopedbuffers);
                    Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(control))))
                },
                CLB() => Err(CreateError { code: 3, message: "Unexpected closing bracket.".to_string() }),
                OPS() => {
                    let mut values: Vec<MutableBuffer> = Vec::new();
                    while let Some(token) = tokens.last() {
                        if let SPC(CLS()) = token {tokens.pop(); break}
                        values.push(read_mutable_buffer(tokens, None)?);
                    }
                    Ok(CreateDirective::WRITE_ARR(values))
                },
                CLS() => Err(CreateError { code: 3, message: "Unexpected closing square bracket.".to_string() }),
                GIA(n) => {
                    let mutbuffer = read_mutable_buffer(tokens, None)?;
                    if let Some(SPC(CLS())) = tokens.last() {
                        tokens.pop();
                        Ok(CreateDirective::READ_IAR(n, mutbuffer))
                    } else {
                        let mut mutbuffers = Vec::new();
                        mutbuffers.push(mutbuffer);
                        while let Some(token) = tokens.last() {
                            if let SPC(CLS()) = token {tokens.pop(); break}
                            mutbuffers.push(read_mutable_buffer(tokens, None)?);
                        }
                        mutbuffers.reverse();
                        Ok(CreateDirective::READ_LIA(n, mutbuffers))
                    }
                },
                OPR() => return Err(CreateError { code: 2, message: "Unexpected ( in source".to_string() }),
                CLR() => return Err(CreateError { code: 2, message: "Unexpected ) in source".to_string() }),
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
                FRN => {
                    let mut identifier: Option<String> = None;
                    if let Some(SPC(SNB(i))) = tokens.last() {
                        identifier = Some(i.clone());
                        tokens.pop();
                    }
                    let array = read_mutable_buffer(tokens, None)?;
                    let control = ForIn::new(array, identifier, read_mutable_buffer(tokens, None)?);
                    Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(control))))
                },
                WHL => {
                    let control = While::new(read_mutable_buffer(tokens, None)?, read_mutable_buffer(tokens, None)?);
                    Ok(CreateDirective::CONTROL(Rc::new(RefCell::new(control))))
                },
                BRK => Ok(CreateDirective::BREAK()),
                RTN => Ok(CreateDirective::RETURN()),
                _ => Err(CreateError { code: 3, message: "Unexpected control flow token found.".to_string() }),
            }
        }
    }
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
            | READ_IAR(_,_)
            | READ_NBF(_)
            | WRITE_BUF(_)
            | WRITE_ARR(_) => {
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
            | WRITE_GNA(_)
            | WRITE_LNB(_)
            | WRITE_LNA(_)
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
            | READ_IAR(_,_)
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