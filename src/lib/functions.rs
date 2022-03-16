use std::rc::Rc;
use std::cell::RefCell;
use super::interpreter::*;
use super::errors::*;

#[derive(Clone, Debug)]
pub struct Function {
    params: Vec<(CreateType, String)>,
    mutbuffer: MutableBuffer,
}

impl Function {
    pub fn new(params: Vec<(CreateType, String)>, mutbuffer: MutableBuffer) -> Self {
        Function { params, mutbuffer }
    }
}

impl Function {
    pub fn evaluate(&self, params: &mut Vec<CreateAny>, environment: &mut Environment, lossy: bool) -> Result<CreateAny, CreateError> {
        let mut priv_scope = Scope::new(Box::new(environment.scope));
        for (index, (param, val)) in self.params.iter().zip(params).enumerate() {
            if !param.0.matches(val) {return Err(CreateError { code: 10, message: format!("Argument {} in function call was mistyped.", index) })}
            priv_scope.insert(param.1.clone(), val.clone());
        }
        let mut partitioned_buffers = PartitionedBuffers::new(environment.buffers);
        match self.mutbuffer.clone().evaluate_clone(&mut Environment { buffers: &mut partitioned_buffers, writers: &mut Writers::new(), scope: &mut priv_scope }, lossy) {
            CreateResult::Ok() => (),
            CreateResult::Err(e) => match e.code {
                12 => (),
                _ => return Err(e),
            }
        }
        Ok(match partitioned_buffers.get_return() {
            Some(v) => *v,
            None => CreateAny::NUL(),
        })
    }
}

#[derive(Clone)]
pub struct FunctionCall {
    params: Vec<MutableBuffer>,
    name: String,
}

impl FunctionCall {
    pub fn new(name: String, params: Vec<MutableBuffer>) -> Self {
        FunctionCall { params, name }
    }

    
}

impl Controller for FunctionCall {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        let function = match environment.scope.get(&self.name) {
            Ok(CreateAny::FUN(f)) => f.clone(),
            Ok(_) => return CreateResult::Err(CreateError { code: 3, message: format!("{} is not a function in the current scope", self.name) }),
            Err(e) => return CreateResult::Err(e),
        };
        let mut params = Vec::new();
        for m in &mut self.params {
            params.push(match m.eval_return(environment, lossy) {
                Ok(Some(v)) => *v,
                Ok(None) => return CreateResult::Err(CreateError { code: 3, message: "Functions cannot have ! values passed as arguments".to_string() }),
                Err(e) => return CreateResult::Err(e),
            });
        }
        match function.evaluate(&mut params, environment, lossy) {
            Ok(CreateAny::NUL()) => (),
            Ok(v) => {write(environment, v, lossy);},
            Err(e) => return CreateResult::Err(e),
        }
        CreateResult::Ok()
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(self.clone()))
    }
}
