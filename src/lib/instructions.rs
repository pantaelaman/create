use crate::lib::interpreter::*;
use crate::lib::errors::*;

pub struct Add {
    left: Option<Buffer>,
    right: Option<Buffer>,
}

impl Add {
    pub fn new() -> Self {
        Add { left: None, right: None }
    }
}

impl Instruction for Add {
    fn evaluate(&self) -> Result<Buffer, CreateError> {
        if let Some(l) = &self.left {
            if let Some(r) = &self.right {
                Ok(l + r)
            } else {Err(CreateError { code: 5, message: "Could not read second value of Add instruction.".to_string() })}
        } else {Err(CreateError {code: 5, message: "Could not read first value of Add instruction".to_string() })}
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

pub struct Sub {
    left: Option<Buffer>,
    right: Option<Buffer>,
}

impl Sub {
    pub fn new() -> Self {
        Sub { left: None, right: None }
    }
}

impl Instruction for Sub {
    fn evaluate(&self) -> Result<Buffer, CreateError> {
        if let Some(l) = &self.left {
            if let Some(r) = &self.right {
                Ok(l - r)
            } else {Err(CreateError { code: 5, message: "Could not read second value of Sub instruction.".to_string() })}
        } else {Err(CreateError {code: 5, message: "Could not read first value of Sub instruction".to_string() })}
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

pub struct Mul {
    left: Option<Buffer>,
    right: Option<Buffer>,
}

impl Mul {
    pub fn new() -> Self {
        Mul { left: None, right: None }
    }
}

impl Instruction for Mul {
    fn evaluate(&self) -> Result<Buffer, CreateError> {
        if let Some(l) = &self.left {
            if let Some(r) = &self.right {
                Ok(l * r)
            } else {Err(CreateError { code: 5, message: "Could not read second value of Mul instruction.".to_string() })}
        } else {Err(CreateError {code: 5, message: "Could not read first value of Mul instruction".to_string() })}
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

pub struct Div {
    left: Option<Buffer>,
    right: Option<Buffer>,
}

impl Div {
    pub fn new() -> Self {
        Div { left: None, right: None }
    }
}

impl Instruction for Div {
    fn evaluate(&self) -> Result<Buffer, CreateError> {
        if let Some(l) = &self.left {
            if let Some(r) = &self.right {
                Ok(l / r)
            } else {Err(CreateError { code: 5, message: "Could not read second value of Div instruction.".to_string() })}
        } else {Err(CreateError {code: 5, message: "Could not read first value of Div instruction".to_string() })}
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

pub struct Mod {
    left: Option<Buffer>,
    right: Option<Buffer>,
}

impl Mod {
    pub fn new() -> Self {
        Mod { left: None, right: None }
    }
}

impl Instruction for Mod {
    fn evaluate(&self) -> Result<Buffer, CreateError> {
        if let Some(l) = &self.left {
            if let Some(r) = &self.right {
                Ok(l % r)
            } else {Err(CreateError { code: 5, message: "Could not read second value of Mod instruction.".to_string() })}
        } else {Err(CreateError {code: 5, message: "Could not read first value of Mod instruction".to_string() })}
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
