pub struct CompilerError {
    code: usize,
    message: String,
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let err_msg = match self.code {
            0 => "Huh, that's a toughie. You've got an error with no error!",
            1 => "There was an internal compiler error; run with -d to see a more detailed report.",
            _ => "Huh, we weren't able to diagnose the issue, but there was an error somewhere in here.",
        };
        write!(f, "{}\n", err_msg)
    }
}

impl std::fmt::Debug for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "A compiler error was thrown. Details follow:\nError Code: {}\nError Message: {}\n", self.code, self.message)
    }
}

impl std::convert::From<std::io::Error> for CompilerError {
    fn from(err: std::io::Error) -> Self {
        let msg = err.to_string();
        CompilerError {code: 1, message: msg}
    }
}