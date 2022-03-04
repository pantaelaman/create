use crate::lib::interpreter::*;
use crate::lib::errors::*;

pub struct BinaryOp {
    left: Option<Buffer>,
    right: Option<Buffer>,
    op: Box<dyn Fn(Buffer, Buffer) -> Buffer>,
}

impl Instruction for BinaryOp {
    fn evaluate(&self) -> Result<Buffer, CreateError> {
        if let Some(l) = self.left {
            if let Some(r) = self.right {
                return Ok((self.op)(l,r));
            }
        }
        Err(CreateError { code: 5, message: "There was an unfilled value within a Binary Operator".to_string() })
    }
    
    fn write(&mut self, value: Buffer) -> CreateResult {
        if let Some(_) = self.left {
            if let Some(_) = self.right {
                CreateResult::Err(CreateError { code: 3, message: "Tried to add a value to a filled instruction".to_string() })
            } else {
                self.right = Some(value);
                CreateResult::Ok()
            }
        } else {
            self.left = Some(value);
            CreateResult::Ok()
        }
    }

    fn is_full(&self) -> Result<bool, CreateError> {
        if let Some(_) = self.left {
            if let Some(_) = self.right {
                Ok(true)
            } else {Ok(false)}
        } else {Ok(false)}
    }
}

impl BinaryOp {
    pub fn new(op: Box<dyn Fn(Buffer, Buffer) -> Buffer>) -> Self {
        BinaryOp { left: None, right: None, op }
    }
}
