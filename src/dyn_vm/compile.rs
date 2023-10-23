
use pest::Parser;
use pest::iterators::Pair;
use pest::error::Error;
use std::{fs, collections::BTreeMap, ops::Index};
use super::{Op, Module};

use crate::lang::*;

use super::*;

#[derive(Debug)]
pub enum LangError {
    NoMain,
    ParserError(Error<Rule>),
    UnknownVar(String),
    UnknownFunction(String),
    VarAlreadyDeclared(String),
}

impl From<pest::error::Error<Rule>> for LangError {
    fn from(value: pest::error::Error<Rule>) -> Self {
        LangError::ParserError(value)
    }
}



#[derive(Debug)]
pub struct ModuleBuilder<'a> {
    code: Vec<Op> ,
    functions: BTreeMap<String,FunctionValue>,
    scope: BTreeMap<&'a str, VarValue>,
    arg_count: usize,
    next_index: usize,
    function_start: usize,
}

impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        ModuleBuilder {
            code: vec![Op::Halt],
            functions: BTreeMap::new(),
            scope: BTreeMap::new(),
            arg_count: 0,
            next_index: 0,
            function_start: 0,
        }
    } 

    pub fn new_frame(&mut self) {
        self.scope = BTreeMap::new();
        self.next_index = 0;
        self.arg_count = 0;
        self.function_start = self.code.len();
    }

    pub fn into_module(self) -> Result<Module, LangError> {
        let Some(main) = self.functions.get("main") else {
            return Err(LangError::NoMain);
        };

        let resulst = Module {
            start: main.offset,
            code: self.code,
            functions: self.functions,
        };

        Ok(resulst)
    }

    fn new_function(&mut self, name: &'a str)  {
        
        let function = FunctionValue {
            name: name.to_string(),
            offset: self.function_start,
            args: self.arg_count,
            vars: self.scope.values().map(|v|(*v).clone()).collect(),
        };

        self.functions.insert(name.to_string(), function);
        
    }

    fn add_op(&mut self, op:Op) {
        self.code.push(op);
    }

    fn new_var(&mut self, name: &'a str) -> Result<usize, LangError> {
        if self.scope.contains_key(name) {
            return Err(LangError::VarAlreadyDeclared(name.to_string()));
        }

        let index = self.next_index;
        self.next_index += 1;

        let var = VarValue {
            name: name.to_string(),
            index,
            var_type: Type::Unknown,
        };
        self.scope.insert(name, var);
        Ok(index)
    }

    fn get_var<'b>(&'b self, name: &str) -> Result<&'b VarValue, LangError> {
        match self.scope.get(name) {
            None => Err(LangError::UnknownVar(name.to_string())),
            Some(var) => Ok(var),
        }
    }
}

pub fn parse_colang_file<'a>(file: &'a str) -> Result<Module, LangError> {
    let data = fs::read_to_string(file).expect("Unable to read file");
    let pairs = LangParser::parse(Rule::program, &data)?;
    let mut builder = ModuleBuilder::new();

    for pair in pairs {
        parse_pair(&mut builder, pair)?;
    }

    let result = builder.into_module()?;
    Ok(result)
}

fn parse_pair<'a>(builder: &mut ModuleBuilder<'a>, pair: Pair<'a, Rule>) -> Result<(), LangError> {
    use Rule::*;

    match pair.as_rule() {
        WHITESPACE
        | number
        | value
        | op
        | expression
        | statment
        | program
        // These rules are silent
        => unreachable!(),
        EOI => { 
            // Noop
        },

        F32 => {
            let v = pair.as_str().parse().unwrap();
            builder.code.push(Op::F32(v));
        },
        F64 => {
            let v = pair.as_str().parse().unwrap();
            builder.code.push(Op::F64(v));
        },
        I32 => {
            let v = pair.as_str().parse().unwrap();
            builder.code.push(Op::I32(v));
        },
        I64 => {
            let v = pair.as_str().parse().unwrap();
            builder.code.push(Op::I64(v));
        },
        U32 => {
            let v = pair.as_str().parse().unwrap();
            builder.code.push(Op::U32(v));
        },
        U64 => {
            let v = pair.as_str().parse().unwrap();
            builder.code.push(Op::U64(v));
        },

        symbol => {
            let name = pair.as_str();
            builder.code.push(Op::Symbol(name.to_string()));
        },

        var => {
            let mut parts = pair.into_inner();
            let name = parts.next().unwrap();
            let var_value = builder.get_var(name.as_str())?;
            builder.add_op(Op::Usize(var_value.index));

            builder.code.push(Op::Load);

        },
        add => {
            builder.code.push(Op::Add);
        },
        sub => {
            //builder.code.push(Op::Sub);
        },
        mul => {
            //builder.code.push(Op::Mul);
        },
        div => {
            //builder.code.push(Op::Div);
        },
        exp => {
            //builder.code.push(Op::Exp);
        },
        opperation => {
            let mut parts = pair.into_inner();

            let first = parts.next().unwrap();
            let operator = parts.next().unwrap();
            let second = parts.next().unwrap();

            parse_pair(builder, first)?;
            parse_pair(builder, second)?;
            parse_pair(builder, operator)?;
        },

        params => {
            let parts = pair.into_inner();
            for p in parts {
                parse_pair(builder, p)?;
            }
        },

        call => {
            let mut parts = pair.into_inner();

            let name = parts.next().unwrap();
            let arguments = parts.next().unwrap();

            parse_pair(builder, arguments)?;
            parse_pair(builder, name)?;
            builder.code.push(Op::GetFn);
            builder.code.push(Op::Call);

  
        },

        declaration => {
            let mut parts = pair.into_inner();

            let l_value = parts.next().unwrap();
            let r_value = parts.next().unwrap();

            parse_pair(builder, r_value)?;
            let name = l_value.as_str();
            let index = builder.new_var(name)?;
            builder.add_op(Op::Usize(index));
            builder.code.push(Op::Store);
        },

        ret => {
            for pair in pair.into_inner() {
                parse_pair(builder, pair)?;
            }
     
            builder.code.push(Op::Return);
        },

   
        args => {
            for arg_n in pair.into_inner() {
                let name = arg_n.as_str();
                let index = builder.new_var(name)?;
                builder.add_op(Op::Usize(index));
                builder.code.push(Op::Store);
            }
        },

        body => {
            for statement_n in pair.into_inner() {
                parse_pair(builder, statement_n)?;
            }
        },

        function => {
            builder.new_frame();

            let mut parts = pair.into_inner();
            
            let fn_name = parts.next().unwrap();
            let fn_args = parts.next().unwrap();
            let fn_body = parts.next().unwrap();

            let name = fn_name.as_str();


            // Process fn args
            parse_pair(builder, fn_args)?;

            // Process statements
            parse_pair(builder, fn_body)?;

            builder.new_function(name);

            builder.code.push(Op::Return);

        }, 
    };
        // program => ... BUG
    Ok(())
}


