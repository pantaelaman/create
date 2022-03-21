use super::tokenizer::*;
use super::functions::*;
use super::interpreter::*;
use super::errors::*;

pub fn read_scope(tokens: &mut Vec<Token>) -> Result<ScopePrototype, CreateError> {
    use Token::*;
    use Special::*;
    let mut scope = ScopePrototype::new();
    while let Some(token) = tokens.pop() {
        match token {
            SPC(SNB(mut n)) => {
                if n.len() > 1 {return Err(CreateError { code: 3, message: "Scopes can only have single level value names".to_string() })}
                scope.insert(n.pop().unwrap(), read_mutable_buffer(tokens, None)?, CreateType::BUF);
            },
            SPC(SNA(mut n)) => {
                if n.len() > 1 {return Err(CreateError { code: 3, message: "Scopes can only have single level value names".to_string() })}
                scope.insert(n.pop().unwrap(), read_mutable_buffer(tokens, None)?, CreateType::ARR);
            },
            SPC(SNF(mut n)) => {
                if n.len() > 1 {return Err(CreateError { code: 3, message: "Scopes can only have single level value names".to_string() })}
                scope.insert(n.pop().unwrap(), read_mutable_buffer(tokens, None)?, CreateType::FUN);
            },
            SPC(SNS(mut n)) => {
                if n.len() > 1 {return Err(CreateError { code: 3, message: "Scopes can only have single level value names".to_string() })}
                scope.insert(n.pop().unwrap(), read_mutable_buffer(tokens, None)?, CreateType::SCP);
            }, 
            SPC(PIP()) => break,
            _ => return Err(CreateError { code: 3, message: "Expected setter or closing pipe in scope declaration".to_string() }),
        }
    }
    Ok(scope)
}

pub fn read_function(tokens: &mut Vec<Token>) -> Result<Function, CreateError> {
    use Token::*;
    use Special::*;
    let mut params: Vec<(CreateType, Identifier)> = Vec::new();
    while let Some(token) = tokens.pop() {
        match token {
            SPC(CLR()) => break,
            SPC(SNB(n)) => params.push((CreateType::BUF, n)),
            SPC(SNA(n)) => params.push((CreateType::ARR, n)),                            
            SPC(SNS(n)) => params.push((CreateType::SCP, n)),
            SPC(SNF(n)) => params.push((CreateType::FUN, n)),
            _ => return Err(CreateError { code: 3, message: "Function arguments must be a form of setter".to_string() }),
        }                   
    }
    let filtered_params = params.into_iter()
        .map(|mut i| {
            Ok((i.0, {
                    if i.1.len() > 1 {return Err(CreateError { code: 3, message: "Function arguments can only be first layer names".to_string() })}
                    else {i.1.pop().unwrap()}
            }))
        })
        .collect::<Result<Vec<(CreateType, String)>, CreateError>>()?;
    let return_type = match tokens.pop().ok_or(CreateError { code: 2, message: "Expected a return type in function declaration".to_string() })? {
        TYP(t) => t,
        _ => return Err(CreateError { code: 3, message: "Expected type statement after function declaration".to_string() }),
    };
    let fun = Function::new(filtered_params, read_mutable_buffer(tokens, None)?, return_type);
    Ok(fun)
}

pub fn read_char(chars: &mut impl Iterator<Item = char>) -> Result<usize, CreateError> {
    let chr = chars.next().ok_or(CreateError { code: 2, message: "Expected char in string".to_string() })?;
    match chr {
        '\\' => {
            let spec = chars.next().ok_or(CreateError { code: 2, message: "Expected char in string".to_string() })?;
            match spec {
                's' => Ok(' ' as usize),
                'n' => Ok('\n' as usize),
                'r' => Ok('\r' as usize),
                c => Ok(c as usize),
            }
        },
        c => Ok(c as usize)
    }
}
