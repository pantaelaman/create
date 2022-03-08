use crate::lib::interpreter::*;
use crate::lib::tokenizer::*;
use crate::lib::errors::*;

pub struct If {
    condition: Buffer,
    tokens: Vec<Token>,
}

impl Controller for If {
    fn run(&mut self, buffers: &mut Buffers, writers: &mut Writers, scope: &mut dyn Scoping) -> CreateResult {
        if self.condition != 0. {
            while let Some(_) = self.tokens.last() {
                match write_token(&mut self.tokens, buffers, writers, scope) {
                    CreateResult::Ok() => (),
                    CreateResult::Err(e) => return CreateResult::Err(e),
                }
            }
        }
        CreateResult::Ok()
    }
}

impl If {
    pub fn new(condition: Buffer, tokens: Vec<Token>) -> Self {
        If { condition, tokens }
    }
}

pub struct IfElse {
    condition: Buffer,
    iftokens: Vec<Token>,
    elsetokens: Vec<Token>,
}

impl Controller for IfElse {
    fn run(&mut self, buffers: &mut Buffers, writers: &mut Writers, scope: &mut dyn Scoping) -> CreateResult {
        if self.condition != 0. {
            while let Some(_) = self.iftokens.last() {
                match write_token(&mut self.iftokens, buffers, writers, scope) {
                    CreateResult::Ok() => (),
                    CreateResult::Err(e) => return CreateResult::Err(e),
                }
            }
        } else {
            while let Some(_) = self.elsetokens.last() {
                match write_token(&mut self.elsetokens, buffers, writers, scope) {
                    CreateResult::Ok() => (),
                    CreateResult::Err(e) => return CreateResult::Err(e),
                }
            }
        }
        CreateResult::Ok()
    }
}

impl IfElse {
    pub fn new(condition: Buffer, iftokens: Vec<Token>, elsetokens: Vec<Token>) -> Self {
        IfElse { condition, iftokens, elsetokens }
    }
}

pub struct For {
    times: Buffer,
    identifier: Option<String>,
    tokens: Vec<Token>,
}

impl Controller for For {
    fn run(&mut self, buffers: &mut Buffers, writers: &mut Writers, scope: &mut dyn Scoping) -> CreateResult {
        'main: for num in 0..(self.times.trunc() as i32) {
            match &self.identifier {
                Some(i) => {scope.insert(i.clone(), CreateAny::BUF(num as Buffer));},
                None => (),
            };
            let mut curtokens = self.tokens.clone();
            while let Some(token) = curtokens.last() {
                if let Token::CFL(ControlFlow::BRK) = token {break 'main}
                match write_token(&mut curtokens, buffers, writers, scope) {
                    CreateResult::Ok() => (),
                    CreateResult::Err(e) => return CreateResult::Err(e),
                }
            }
        }
        CreateResult::Ok()
    }
}

impl For {
    pub fn new(times: Buffer, identifier: Option<String>, tokens: Vec<Token>) -> Self {
        For { times, identifier, tokens }
    }
}

pub struct While {
    condition: MutableBuffer,
    tokens: Vec<Token>,
}

impl Controller for While {
    fn run(&mut self, buffers: &mut Buffers, writers: &mut Writers, scope: &mut dyn Scoping) -> CreateResult {
        'main: while match self.condition.clone().evaluate(buffers, writers, scope) {
            CreateResult::Ok() => true,
            CreateResult::Err(_) => false,
        } {
            let mut curtokens = self.tokens.clone();
            while let Some(token) = curtokens.last() {
                if let Token::CFL(ControlFlow::BRK) = token {break 'main}
                match write_token(&mut curtokens, buffers, writers, scope) {
                    CreateResult::Ok() => (),
                    CreateResult::Err(e) => return CreateResult::Err(e),
                }
            }
        }
        CreateResult::Ok()
    }
}

impl While {
    pub fn new(mut tokens: Vec<Token>) -> Result<Self, CreateError> {
        let condition = read_mutable_buffer(&mut tokens)?;
        Ok(While { condition, tokens })
    }
}

pub struct Scoped {
    mutbuffers: Vec<MutableBuffer>,
}

impl Controller for Scoped {
    fn run(&mut self, buffers: &mut Buffers, writers: &mut Writers, scope: &mut dyn Scoping) -> CreateResult {
        let mut scope = Scope::new(Box::new(scope));
        println!("{:?}", self.mutbuffers);
        while let Some(mut mutbuffer) = self.mutbuffers.pop() {
            match mutbuffer.evaluate(buffers, writers, &mut scope) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
        }
        CreateResult::Ok()
    }
}

impl Scoped {
    pub fn new(mutbuffers: Vec<MutableBuffer>) -> Self {
        Scoped { mutbuffers }
    }
}
