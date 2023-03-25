
use pest::Parser;
pub use pest::error::Error;
use std::fs;

struct Symbol<'a>(&'a str);

pub enum Value {
    U32(u32),
}

pub enum Op {
    Plus,
}

pub enum Expression<'a> {
    Value(Value),
    Op(Op, Box<Expression<'a>>, Box<Expression<'a>>),
    Call(Symbol<'a>, Vec<Symbol<'a>>)
}

pub enum Statement<'a> {
    Var(Symbol<'a>, Expression<'a>),
    Ret(Expression<'a>),
    Expression(Expression<'a>),
}

pub enum TopLevel<'a> {
    Function(Symbol<'a>, Vec<Symbol<'a>>, Vec<Statement<'a>>),
}


#[derive(Parser)]
#[grammar = "lang/grammar.pest"]
pub struct LangParser;

pub fn parse_colang_file(file: &str) -> Result<Vec<TopLevel>, Error<Rule>> {
    let data = fs::read_to_string(file).expect("Unable to read file");
    dbg!(&data);
    let program = LangParser::parse(Rule::program, &data)?.next().unwrap();

    dbg!(program);
    panic!();
    Ok(Vec::new())
}

#[cfg(test)]
mod test;