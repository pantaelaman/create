#[derive(Clone)]
pub struct CreateError {
    pub code: usize,
    pub message: String,
}

impl std::fmt::Display for CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let err_msg = match self.code {
            0 => "Huh, that's a toughie. You've got an error with no error!",
            1 => "There was an internal compiler error; run with -d to see a more detailed report.",
            2 => "There was improper formatting in the source code.",
            3 => "There was a syntax error in the source code.",
            4 => "We were not able to access the buffer at a specified index.",
            5 => "There was an unfilled value in an instruction.",
            6 => "There was an uninstantiated named buffer.",
            7 => "Tried to write incompatible type.",
            8 => "Tried to print a floating point char.",
            9 => "Improper condition for control flow statement.",
            10 => "Mistyped argument for function.",
            11 => "Unexpected break statement.",
            12 => "Unexpected return statement.",
            usize::MAX => "Something went wrong.",
            _ => "Huh, we weren't able to diagnose the issue, but there was an error somewhere in here.",
        };
        write!(f, "{}\n", err_msg)
    }
}

impl std::fmt::Debug for CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "An error was thrown. Details follow:\nError Code: {}\nError Message: {}\n", self.code, self.message)
    }
}

impl std::convert::From<std::io::Error> for CreateError {
    fn from(err: std::io::Error) -> Self {
        let msg = err.to_string();
        CreateError {code: 1, message: msg}
    }
}

pub enum CreateResult {
    Ok(),
    Err(CreateError),
}
