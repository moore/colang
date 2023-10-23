extern crate pest;
#[macro_use]
extern crate pest_derive;

mod table;
use crate::table::*;

mod typed_vm;

mod dyn_vm;
//mod sym_vm;

mod lang;

#[derive(Debug, Clone)]
struct Var {
    name: String,
    var_type: Type,
}

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    args: Vec<Type>,
    vars: Vec<Var>,
}

#[derive(Debug, Clone)]
pub enum Type {
    None,
    Unknown,
    Usize,
    F32,
    F64,
    U32,
    U64,
    I32,
    I64,
    Symbol,
    StringRef,
    Bool,
    Struct(Vec<Type>),
    Table,
    Cursor,
    Function(Function),
}






