use std::rc::Rc;
use std::cell::RefCell;
use crate::lib::interpreter::*;
use crate::lib::errors::*;

pub struct If {
    condition: MutableBuffer,
    mutbuffer: MutableBuffer,
}

impl Controller for If {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        match self.condition.evaluate(environment, lossy) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => return CreateResult::Err(e),
        }
        if match environment.buffers.get_buf(0) {
            Some(b) => *b,
            None => return CreateResult::Err(CreateError { code: 5, message: "If condition did not return buffer, and no buffers were found.".to_string() })
        } != 0. {
            self.mutbuffer.evaluate(environment, lossy)
        } else {
            CreateResult::Ok()
        }
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(If::new(self.condition.clone(), self.mutbuffer.clone())))
    }
}

impl If {
    pub fn new(condition: MutableBuffer, mutbuffer: MutableBuffer) -> Self {
        If { condition, mutbuffer }
    }
}

pub struct IfElse {
    condition: MutableBuffer,
    ifmutbuffer: MutableBuffer,
    elsemutbuffer: MutableBuffer,
}

impl Controller for IfElse {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        match self.condition.evaluate(environment, lossy) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => return CreateResult::Err(e),
        }
        if match environment.buffers.get_buf(0) {
            Some(b) => *b,
            None => return CreateResult::Err(CreateError { code: 3, message: "IfElse condition did not return a buffer, and no buffer was found.".to_string() }),
        } != 0. {
            self.ifmutbuffer.evaluate(environment, lossy)
        } else {
            self.elsemutbuffer.evaluate(environment, lossy)
        }
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(IfElse::new(self.condition.clone(), self.ifmutbuffer.clone(), self.elsemutbuffer.clone())))
    }
}

impl IfElse {
    pub fn new(condition: MutableBuffer, ifmutbuffer: MutableBuffer, elsemutbuffer: MutableBuffer) -> Self {
        IfElse { condition, ifmutbuffer, elsemutbuffer }
    }
}

pub struct For {
    times: MutableBuffer,
    identifier: Option<String>,
    mutbuffer: MutableBuffer,
}

impl Controller for For {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        match self.times.evaluate(environment, lossy) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => return CreateResult::Err(e),
        }
        let mut scope = Scope::new(Box::new(environment.scope));        
        for num in 0..(match environment.buffers.get_buf(0) {
            Some(b) => b,
            None => return CreateResult::Err(CreateError { code: 3, message: "For loop condition did not return value, and no value was found in buffer".to_string() }),
        }.trunc() as i32) {
            match &self.identifier {
                Some(i) => {scope.insert_locally(i.clone(), CreateAny::BUF(num as Buffer));},
                None => (),
            };
            match self.mutbuffer.clone().evaluate_clone(&mut Environment { buffers: environment.buffers, writers: environment.writers, scope: &mut scope }, false) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            };
        }
        CreateResult::Ok()
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(For::new(self.times.clone(), self.identifier.clone(), self.mutbuffer.clone())))
    }
}

impl For {
    pub fn new(times: MutableBuffer, identifier: Option<String>, mutbuffer: MutableBuffer) -> Self {
        For { times, identifier, mutbuffer }
    }
}

pub struct ForIn {
    value: MutableBuffer,
    identifier: Option<String>,
    mutbuffer: MutableBuffer,
}

impl Controller for ForIn {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        match self.value.evaluate(environment, lossy) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => return CreateResult::Err(e),
        }
        let array = match environment.buffers.get_arr(0) {
            Some(a) => *a,
            None => return CreateResult::Err(CreateError { code: 3, message: "ForIn loop condition did not return array, and array was found in buffer".to_string() })
        };
        let mut scope = Scope::new(Box::new(environment.scope));
        for value in array {
            match &self.identifier {
                Some(i) => {scope.insert_locally(i.clone(), value);},
                None => (),
            }
            match self.mutbuffer.clone().evaluate_clone(&mut Environment { buffers: environment.buffers, writers: environment.writers, scope: &mut scope }, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
        }
        CreateResult::Ok()
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(ForIn::new(self.value.clone(), self.identifier.clone(), self.mutbuffer.clone())))
    }
}

impl ForIn {
    pub fn new(value: MutableBuffer, identifier: Option<String>, mutbuffer: MutableBuffer) -> Self {
        ForIn { value, identifier, mutbuffer }
    }
}

pub struct While {
    condition: MutableBuffer,
    mutbuffer: MutableBuffer,
}

impl Controller for While {
    fn run(&mut self, environment: &mut Environment, _lossy: bool) -> CreateResult {
        
        while match self.condition.clone().evaluate_clone(environment, false) {
            CreateResult::Ok() => match environment.buffers.get_buf(0) {
                Some(b) => *b,
                None => return CreateResult::Err(CreateError { code: 3, message: "While condition did not return buffer, and no buffer was found.".to_string() })
            },
            CreateResult::Err(e) => return CreateResult::Err(e),
        } != 0. {
            self.mutbuffer.clone().evaluate_clone(environment, false);
        }
        CreateResult::Ok()
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(While::new(self.condition.clone(), self.mutbuffer.clone())))
    }
}

impl While {
    pub fn new(condition: MutableBuffer, mutbuffer: MutableBuffer) -> Self {
        While { condition, mutbuffer }
    }
}

pub struct Scoped {
    mutbuffers: Vec<MutableBuffer>,
}

impl Controller for Scoped {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        let mut scope = Scope::new(Box::new(environment.scope));
        while let Some(mut mutbuffer) = self.mutbuffers.pop() {
            match mutbuffer.evaluate(&mut Environment { buffers: environment.buffers, writers: environment.writers, scope: &mut scope }, lossy) {
                CreateResult::Ok() => (),
                CreateResult::Err(e) => return CreateResult::Err(e),
            }
        }
        CreateResult::Ok()
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(Scoped::new(self.mutbuffers.clone())))
    }
}

impl Scoped {
    pub fn new(mutbuffers: Vec<MutableBuffer>) -> Self {
        Scoped { mutbuffers }
    }
}
