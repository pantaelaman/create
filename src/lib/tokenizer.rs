use crate::lib::errors;

#[derive(Debug, Clone)]
pub enum Token {
    CMD(Command),
    NUM(f32),
    SPC(Special),
}

#[derive(Debug, Clone)]
pub enum Command {
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    POW,
}

#[derive(Debug, Clone)]
pub enum Special {
    BUF(),
    IBF(usize),
}

pub fn tokenize(data: &str) -> Result<Vec<Token>, errors::CreateError> {
    let instructions = data.split('\n').collect::<Vec<&str>>();
    let mut tokens: Vec<Token> = Vec::new();
    for (line,instruction) in instructions.iter().enumerate() {
        let raw_tokens = instruction.split(char::is_whitespace).filter(|el| {el != &""}).collect::<Vec<&str>>();
        for (chr, raw_token) in raw_tokens.iter().enumerate() {
            use Token::*;
            use Command::*;
            use Special::*;
            match raw_token {
                &"-" => tokens.push(CMD(SUB)),
                &"+" => tokens.push(CMD(ADD)),
                &"*" => tokens.push(CMD(MUL)),
                &"/" => tokens.push(CMD(DIV)),
                &"%" => tokens.push(CMD(MOD)),
                &"^" => tokens.push(CMD(POW)),
                &"~" => tokens.push(SPC(BUF())),
                _ => match raw_token.parse::<f32>() {
                    Ok(v) => tokens.push(NUM(v)),
                    Err(_) => { // not a valid number
                        if raw_token.chars().nth(0).unwrap() == '~' {
                            match raw_token[1..].parse::<usize>() {
                                Ok(v) => tokens.push(SPC(IBF(v))),
                                Err(_) => return Err(errors::CreateError{code: 2, message: format!("Improper indexed buffer definition at line {}, char {}", line, chr)})
                            }
                        }
                    },
                },
            }
        }
    }
    return Ok(tokens);
}