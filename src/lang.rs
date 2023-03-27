
use pest::Parser;
use pest::iterators::Pair;
use pest::error::Error;
use std::{fs, collections::BTreeMap};
use crate::typed_vm::{VmError, Module};
use crate::table::FnTable;

use super::{Op,Value,Type};



#[derive(Parser)]
#[grammar = "lang/grammar.pest"]
pub struct LangParser;

#[derive(Debug)]
pub enum LangError {
    NoMain,
    ParserError(Error<Rule>)
}

impl From<pest::error::Error<Rule>> for LangError {
    fn from(value: pest::error::Error<Rule>) -> Self {
        LangError::ParserError(value)
    }
}

pub struct ModuleBuilder<'a> {
    code: Vec<Op> ,
    functions: BTreeMap<&'a str,usize>,
}

impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        ModuleBuilder {
            code: Vec::new(),
            functions: BTreeMap::new(),
        }
    } 

    pub fn into_module(self) -> Result<Module, LangError> {
        let Some(start) = self.functions.get("main") else {
            return Err(LangError::NoMain);
        };


        let resulst = Module {
            start: *start,
            code: self.code,
            functions: FnTable::new(), //BOOG
            types: BTreeMap::new(),
        };

        Ok(resulst)
    }
}

pub fn parse_colang_file<'a>(file: &'a str) -> Result<Module, LangError> {
    let data = fs::read_to_string(file).expect("Unable to read file");
    let pair = LangParser::parse(Rule::program, &data)?.next().unwrap();
    let mut builder = ModuleBuilder::new();

    parse_pair(&mut builder, pair);

    let result = builder.into_module()?;
    Ok(result)
}

fn parse_pair<'a>(builder: &mut ModuleBuilder<'a>, pair: Pair<'a, Rule>) {
    use Rule::*;

    match pair.as_rule() {
        WHITESPACE
        | EOI
        | expression
        | statment
        | program
        | number
        | value
        | op
        => unreachable!(),
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

        symbol => {todo!()},
        var => {todo!()},
        add => {
            builder.code.push(Op::Add);
        },
        sub => {todo!()},
        mul => {todo!()},
        div => {todo!()},
        exp => {todo!()},
        opperation => {
            let mut parts = pair.into_inner();

            let first = parts.next().unwrap();
            let operator = parts.next().unwrap();
            let second = parts.next().unwrap();

            parse_pair(builder, first);
            parse_pair(builder, second);
            parse_pair(builder, operator);
        },
        call => {todo!()},
        declaration => {todo!()},
        ret => {todo!()},
        args => {todo!()},
        body => {todo!()},
        function => {
            let mut parts = pair.into_inner();

            let fn_name = parts.next().unwrap();
            let fn_args = parts.next().unwrap(); // TODO
            let fn_body = parts.next().unwrap();

            let name = fn_name.as_str();
            let index = builder.code.len();
            builder.functions.insert(name, index);

            for statement_n in fn_body.into_inner() {
                parse_pair(builder, statement_n);
            }
            builder.code.push(Op::Halt);

        }, 
    };
    
}

#[cfg(test)]
mod test;