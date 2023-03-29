extern crate pest;
#[macro_use]
extern crate pest_derive;

mod table;
use crate::table::*;

mod typed_vm;
use crate::typed_vm::*;

mod lang;

#[derive(Debug, Clone)]
pub enum Type {
    None,
    Usize,
    U32,
    StringRef,
    Bool,
    Struct(u32),
    Table,
    Cursor,
    Function(u32),
}


#[derive(Debug)]
pub enum Value {
    None,
    Usize(usize),
    F32(f32),
    F64(f64),
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    StringRef{
        index: usize,
    },
    Bool(bool),
    Struct {
        field_count: usize,
    }, 
    Table (TableTypes),
    Cursor(CursorTypes),
    Function {
        ptr: usize,
    },
}

#[derive(Debug)]
pub enum Op {

    /// ( -- ): Do nothing
    Noop,

    /// ( -- ): Stops the Vm
    Halt,

    /// (Value -- ): Remove the top value of the stack   
    Pop,

    /// (Value, Value -- Value, Value): Swap the top to value of the stack.
    Swap,

    /// (Value -- Value, Value): Copy the top value of the stack.
    Copy,

    /// (usize -- Value): Copy the value from down the stack with the depth
    ///  provided by the usize
    CopyFrom,

    /// ( usize -- Value ): Copy the var from frame offset usize to the top
    /// of the stack.
    Load,


    /// ( usize Value -- Value ): Write usize frame offset with the Value.
    /// pops the written value off the top of the stack.
    Store,


    /// (Usize, Usize, Function -- ): Call the given function ref. The first 
    /// argument is the number of arguments which causes, and the second value is the number
    /// of returned values.
    Call,

    /// ( -- )
    Return,

    /// (Usize -- Table) Construct a table with given type index.
    Table,

    /// (Struct, Table -- Cursor) Querying a table using the Struct
    /// that matches the table type to constrain the query. Any Struct
    /// fields with a value of None will be considered to be free and 
    /// not constrain the query.
    Query,

    /// (Cursor -- Cursor, bool): Returns true if the cursor is
    /// at a record which matches the Query
    Found,

    /// (Cursor -- Cursor, Struct): Read the record at the Cursor
    Read,

    /// (Cursor, Struct-- Cursor): Insert in to the table 
    /// creating a new record in the position that fallows
    /// the current Cursor location. Returns a cursor at the 
    /// record. 
    Insert,

    /// (Cursor, Struct-- Cursor): Replace the record at the cursor
    /// with Struct from the stack.
    Update,

    /// (Cursor -- Cursor): Remove the record at the cursor, returning
    /// a cursor at the next record.
    Delete,

    /// (Cursor -- Cursor): Advance the cursor to the next record.
    Advance,

    /// (Cursor -- Table): Closes the cursor returning the table.
    Close,

    /// ( -- None): Push None on to the stack.
    None,

    /// ( -- Fn):: Push a function pointer on the stack.
    Fn(usize),

    /// ( -- U32): Push a U32 on to the stack.
    F32(f32),

    /// ( -- U64): Push a U64 on to the stack.
    F64(f64),

    /// ( -- U32): Push a U32 on to the stack.
    I32(i32),

    /// ( -- U64): Push a U64 on to the stack.
    I64(i64),

    /// ( -- U32): Push a U32 on to the stack.
    U32(u32),

    /// ( -- U64): Push a U64 on to the stack.
    U64(u64),

    /// ( - usize): Push a usize on to the stack.
    Usize(usize),

    /// ( -- Value::Bool): Push a Value::Bool on to the stack
    Bool(bool),

    /// (Size .. -- Struct): Construct a new Struct
    /// consuming stack values as defined by the U32
    Struct,    

    /// (Number<T>, Number<T> --Number<T>): Add two numbers of a matching type
    /// and put the result with the same type on the stack.
    Add,
}







