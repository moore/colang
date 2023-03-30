
use pest::Parser;
use pest::iterators::Pair;
use pest::error::Error;
use std::{fs, collections::BTreeMap};
use crate::typed_vm::{Module};
use crate::table::FnTable;

use super::{Op};

use crate::lang::*;


#[derive(Debug)]
pub enum LangError {
    NoMain,
    ParserError(Error<Rule>),
    UnknownVar(String),
    UnknownFunction(String),
}

impl From<pest::error::Error<Rule>> for LangError {
    fn from(value: pest::error::Error<Rule>) -> Self {
        LangError::ParserError(value)
    }
}

#[derive(Debug)]
 struct FnType {
    index: usize,
    arg_count: usize,
    frame_size: usize,
 }

#[derive(Debug)]
pub struct ModuleBuilder<'a> {
    code: Vec<Op> ,
    functions: BTreeMap<&'a str,FnType>,
    scope: BTreeMap<&'a str, usize>,
    frame_size: usize,
}

impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        ModuleBuilder {
            code: vec![Op::Halt],
            functions: BTreeMap::new(),
            scope: BTreeMap::new(),
            frame_size: 0,
        }
    } 

    pub fn new_frame(&mut self) {
        self.scope = BTreeMap::new();
        self.frame_size = 0;
    }

    pub fn into_module(self) -> Result<Module, LangError> {
        let Some(fn_type) = self.functions.get("main") else {
            return Err(LangError::NoMain);
        };

        let resulst = Module {
            start: fn_type.index,
            code: self.code,
            functions: FnTable::new(), //BOOG
            types: BTreeMap::new(),
        };

        Ok(resulst)
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
            // (Rule::F32(s:str)) => Op::F32(parse(str))
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
            //Noop
        },
        var => {
            let name = pair.as_str();
            let Some(offset) = builder.scope.get(name) else {
                return Err(LangError::UnknownVar(name.to_string()));
            };

            builder.code.push(Op::Usize(*offset));
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

            let sym = parts.next().unwrap();
            let arguments = parts.next().unwrap();

            for a in arguments.into_inner() {
                parse_pair(builder, a)?;
            }

            let name = sym.as_str();

            let Some(fn_info) = builder.functions.get(name) else {
                return Err(LangError::UnknownFunction(name.to_string()));
            };

            builder.code.push(Op::Usize(1)); // BOOG
            builder.code.push(Op::Usize(fn_info.arg_count));
            builder.code.push(Op::Fn(fn_info.index));
            builder.code.push(Op::Call);

  
        },

        declaration => {
            let mut parts = pair.into_inner();

            let l_value = parts.next().unwrap();
            let r_value = parts.next().unwrap();

            parse_pair(builder, r_value)?;

            let name = l_value.as_str();

            let offset = builder.frame_size;
            builder.frame_size += 1;

            builder.scope.insert(name, offset);

            builder.code.push(Op::Usize(offset));
            builder.code.push(Op::Store);
        },

        ret => {
            for pair in pair.into_inner() {
                parse_pair(builder, pair)?;
            }
            builder.code.push(Op::Return);
        },
        args => {todo!()},
        body => {todo!()},
        function => {
            builder.new_frame();

            let mut parts = pair.into_inner();

            let fn_name = parts.next().unwrap();
            let fn_args = parts.next().unwrap();
            let fn_body = parts.next().unwrap();

            let index = builder.code.len();

            // Process fn args
            let mut arg_count = 0;
            for arg_n in fn_args.into_inner() {
                let name = arg_n.as_str();
                let offset = builder.frame_size;
                arg_count += 1;
                builder.frame_size += 1;
                builder.scope.insert(name, offset);
            }

            // Process statements
            for statement_n in fn_body.into_inner() {
                parse_pair(builder, statement_n)?;
            }
            builder.code.push(Op::Return);


            // allocate space on the stack for vars.
            let var_count = builder.frame_size - arg_count;

            for _  in 0..var_count {
                builder.code.insert(index, Op::None);
            }

            // Process fn name

            let name = fn_name.as_str();
            let fn_type = FnType { 
                index,
                arg_count,
                frame_size: builder.frame_size,
            };
            builder.functions.insert(name, fn_type);

        }, 
    };
    Ok(())
}

