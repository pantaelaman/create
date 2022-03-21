use std::rc::Rc;
use std::cell::RefCell;
use super::interpreter::*;
use super::errors::*;

#[derive(Clone, Debug)]
pub struct Function {
    params: Box<Vec<(CreateType, String)>>,
    mutbuffer: Box<MutableBuffer>,
    returntype: Box<CreateType>,
}

impl Function {
    pub fn new(params: Vec<(CreateType, String)>, mutbuffer: MutableBuffer, returntype: CreateType) -> Self {
        Function { params: Box::new(params), mutbuffer: Box::new(mutbuffer), returntype: Box::new(returntype)}
    }

    pub fn evaluate(&self, params: &mut Vec<CreateAny>, environment: &mut Environment, lossy: bool) -> Result<CreateAny, CreateError> {
        let mut priv_scope = Scope::new(environment.scope);
        for (index, (param, val)) in self.params.iter().zip(params).enumerate() {
            if !param.0.matches(val) {return Err(CreateError { code: 10, message: format!("Argument {} in function call was mistyped (type {:?}), should be {:?}", index, val.get_type(), param.0) })}
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
        let r = partitioned_buffers.get_return().unwrap_or(Box::new(CreateAny::NUL()));
        if self.returntype.matches(&*r) {
            Ok(*r)
        } else {
            Err(CreateError { code: 3, message: format!("Improper function return (expected {:?})", self.returntype) })
        }
    }
}

#[derive(Clone)]
pub struct FunctionCall {
    params: Vec<MutableBuffer>,
    name: Identifier,
}

impl FunctionCall {
    pub fn new(name: Identifier, params: Vec<MutableBuffer>) -> Self {
        FunctionCall { params, name }
    }

    fn get_function<'a>(&self, environment: &'a Environment) -> Result<&'a Function, CreateError> {
        match resolve_identifier_immut(&self.name, environment.scope) {
            Ok(CreateAny::FUN(f)) => Ok(&*f),
            Ok(_) => return Err(CreateError { code: 3, message: format!("{:?} is not a function in the current scope", self.name) }),
            Err(e) => return Err(e),
        }
    }
}

impl Controller for FunctionCall {
    fn run(&mut self, environment: &mut Environment, lossy: bool) -> CreateResult {
        let function = match self.get_function(environment) {
            Ok(f) => f.clone(),
            Err(e) => return CreateResult::Err(e),
        };
        let mut params = Vec::new();
        for m in &mut self.params {
            params.push(match m.eval_clone_return(environment, lossy) {
                Ok(Some(v)) => *v,
                Ok(None) => return CreateResult::Err(CreateError { code: 3, message: "Functions cannot have none values passed as arguments".to_string() }),
                Err(e) => return CreateResult::Err(e),
            });
        }
        params.reverse();
        match function.evaluate(&mut params, environment, lossy) {
            Ok(v) => {write(environment, v, lossy);},
            Err(e) => return CreateResult::Err(e),
        }
        CreateResult::Ok()
    }

    fn clone_cfl(&self) -> Rc<RefCell<dyn Controller>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn return_count(&self) -> usize {1}
}
