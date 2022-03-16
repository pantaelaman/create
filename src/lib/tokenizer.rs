use regex::Regex;
use crate::lib::errors;

#[derive(Debug, Clone)]
pub enum Token {
    CMD(Command),
    NUM(f32),
    SPC(Special),
    CFL(ControlFlow),
}

#[derive(Debug, Clone)]
pub enum Command {
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    POW,
    SIN,
    COS,
    TAN,
    ASN,
    ACS,
    ATN,
    SQT,
    CBT,
    EQU,
    NOT,
    GTH,
    LTH,
    ORR,
    AND,
    PNT,
    PTC,
}

#[derive(Debug, Clone)]
pub enum Special {
    RMB(),
    BUF(),
    IBF(usize),
    SNB(String),
    SNA(String),
    SGB(String),
    SGA(String),
    SLB(String),
    SLA(String),
    GIA(String),
    GNB(String),
    FNC(String),
    FUN(String),
    OPB(),
    CLB(),
    OPS(),
    CLS(),
    OPR(),
    CLR(),
}

#[derive(Debug, Clone)]
pub enum ControlFlow {
    IFF,
    ELS,
    FOR,
    FRN,
    WHL,
    BRK,
    RTN,
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
            use ControlFlow::*;
            use std::f32::consts::*;
            tokens.push(match raw_token {
                // Basic Operations
                &"-" => CMD(SUB),
                &"+" => CMD(ADD),
                &"*" => CMD(MUL),
                &"/" => CMD(DIV),
                &"%" => CMD(MOD),
                &"^" => CMD(POW),
                // Trig
                &"sin" => CMD(SIN),
                &"cos" => CMD(COS),
                &"tan" => CMD(TAN),
                &"asin" => CMD(ASN),
                &"acos" => CMD(ACS),
                &"atan" => CMD(ATN),
                // Roots
                &"sqrt" => CMD(SQT),
                &"cbrt" => CMD(CBT),
                // Constants
                &"pi" => NUM(PI),
                // Boolean
                &"==" => CMD(EQU),
                &"!" => CMD(NOT),
                &">" => CMD(GTH),
                &"<" => CMD(LTH),
                &"||" => CMD(ORR),
                &"&&" => CMD(AND),
                // Output
                &"." => CMD(PNT),
                &"," => CMD(PTC),
                // Control Flow
                &"if" => CFL(IFF),
                &"else" => CFL(ELS),
                &"for" => CFL(FOR),
                &"forin" => CFL(FRN),
                &"while" => CFL(WHL),
                &"break" => CFL(BRK),
                &"return" => CFL(RTN),
                // Scoping
                &"{" => SPC(OPB()),
                &"}" => SPC(CLB()),
                // Array
                &"[" => SPC(OPS()),
                &"]" => SPC(CLS()),
                // Function
                &"(" => SPC(OPR()),
                &")" => SPC(CLR()),
                // Buffer
                &";" => SPC(RMB()),
                &"~" => SPC(BUF()),
                _ => match raw_token.parse::<f32>() {
                    Ok(v) => NUM(v),
                    Err(_) => { // not a valid number
                        match raw_token.chars().nth(0).unwrap() {
                            '~' => match raw_token[1..].parse::<usize>() {
                                Ok(v) => SPC(IBF(v)),
                                Err(_) => {
                                    if Regex::new(r"^\w+$").unwrap().is_match(&raw_token[1..]) {
                                        SPC(GNB(raw_token[1..].to_string()))
                                    } else if Regex::new(r"^\w+\[$").unwrap().is_match(&raw_token[1..]) {
                                        SPC(GIA(raw_token[1..(raw_token.len()-1)].to_string()))
                                    } else if Regex::new(r"^\w+\($").unwrap().is_match(&raw_token[1..]) {
                                        SPC(FNC(raw_token[1..(raw_token.len()-1)].to_string()))
                                    } else {
                                        return Err(errors::CreateError{ code: 2, message: format!("Could not read name or index of buffer at line {}, char {}", line, chr) });
                                    }
                                }
                            },
                            '=' => {
                                if Regex::new(r"^\w+$").unwrap().is_match(&raw_token[1..]) {
                                    SPC(SNB(raw_token[1..].to_string()))
                                } else if Regex::new(r"^\[\]\w+$").unwrap().is_match(&raw_token[1..]) {
                                    SPC(SNA(raw_token[3..].to_string()))
                                } else if Regex::new(r"^\(\)\w+$").unwrap().is_match(&raw_token[1..]) {
                                    SPC(FUN(raw_token[3..].to_string()))
                                } else {
                                    return Err(errors::CreateError{ code: 2, message: format!("Invalid name for setting a named buffer at line {}, char {}", line, chr)})
                                }
                            },
                            'g' => {
                                if Regex::new(r"^=\w+$").unwrap().is_match(&raw_token[1..]) {
                                    SPC(SGB(raw_token[2..].to_string()))
                                } else if Regex::new(r"^=\[\]\w+$").unwrap().is_match(&raw_token[1..]) {
                                    SPC(SGA(raw_token[4..].to_string()))
                                } else {
                                    return Err(errors::CreateError{ code: 2, message: format!("Invalid token {} at line {}, char {}", raw_token, line, chr)})
                                }
                            },
                            'l' => {
                                if Regex::new(r"^=\w+$").unwrap().is_match(&raw_token[1..]) {
                                    SPC(SLB(raw_token[2..].to_string()))
                                } else if Regex::new(r"^=\[\]\w+$").unwrap().is_match(&raw_token[1..]) {
                                    SPC(SLA(raw_token[4..].to_string()))
                                } else {
                                    return Err(errors::CreateError{ code: 2, message: format!("Invalid token {} at line {}, char {}", raw_token, line, chr)})
                                }
                            }
                            _ => return Err(errors::CreateError{ code: 2, message: format!("Unrecognized token {} at line {}, char {}", raw_token, line, chr) })
                        }
                    },
                },
            });
        }
    }
    return Ok(tokens);
}