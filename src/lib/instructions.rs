use std::rc::Rc;
use std::cell::RefCell;
use crate::lib::interpreter::*;
use crate::lib::errors::*;

pub struct BinaryOp {
    left: Option<Buffer>,
    right: Option<Buffer>,
    op: fn(Buffer, Buffer) -> Buffer,
}

impl Instruction for BinaryOp {
    fn evaluate(&mut self, lossy: bool) -> Result<Buffer, CreateError> {
        if let Some(l) = self.left {
            if let Some(r) = self.right {
                return Ok((self.op)(l,r));
            }
        }
        Err(CreateError { code: 5, message: "There was an unfilled value within a Binary Operator".to_string() })
    }
    
    fn write_buffer(&mut self, value: Buffer) -> CreateResult {
        if let Some(_) = self.left {
            if let Some(_) = self.right {
                CreateResult::Err(CreateError { code: 3, message: "Tried to add a value to a filled Binary Operator".to_string() })
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

    fn capacity(&self) -> Result<usize, CreateError> {Ok(2)}

    fn clone_ins(&self) -> Rc<RefCell<dyn Instruction>> {
        Rc::new(RefCell::new(BinaryOp { left: self.left.clone(), right: self.right.clone(), op: self.op.clone() }))
    }
}

impl BinaryOp {
    pub fn new(op: fn(Buffer, Buffer) -> Buffer) -> Self {
        BinaryOp { left: None, right: None, op }
    }
}

pub struct UnaryOp {
    value: Option<Buffer>,
    op: fn(Buffer) -> Buffer,
}

impl Instruction for UnaryOp {
    fn evaluate(&mut self, lossy: bool) -> Result<Buffer, CreateError> {
        if let Some(v) = self.value {
            Ok((self.op)(v))
        } else {
            Err(CreateError { code: 5, message: "There was an unfilled value in a Unary Operator".to_string() })
        }
    }

    fn write_buffer(&mut self, val: Buffer) -> CreateResult {
        if let Some(_) = self.value {
            CreateResult::Err(CreateError { code: 3, message: "Tried to add a value to a filled Unary Operator".to_string() })
        } else {
            self.value = Some(val);
            CreateResult::Ok()
        }
    }

    fn is_full(&self) -> Result<bool, CreateError> {
        if let Some(_) = self.value {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn capacity(&self) -> Result<usize, CreateError> {Ok(1)}

    fn clone_ins(&self) -> Rc<RefCell<dyn Instruction>> {
        Rc::new(RefCell::new(UnaryOp { value: self.value.clone(), op: self.op.clone() }))
    }
}

impl UnaryOp {
    pub fn new(op: fn(Buffer) -> Buffer) -> Self {
        UnaryOp { value: None, op }
    }
}
