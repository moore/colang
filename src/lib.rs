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
    U32(u32),
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

    /// (Function, U32, U32 -- ): Call the given function ref, it is 
    /// expect to consume the number of args given by the first u32 
    /// and return the number specified byt the second. 
    Call,

    /// ( -- )
    Return,

    /// (u32 -- Table) Construct a table with given type index.
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

    /// ( -- U32): Push a U32 on to the stack.
    U32(u32),

    /// ( - usize): Push a usize on to the stack.
    Usize(usize),

    /// ( -- Value::Bool): Push a Value::Bool on to the stack
    Bool(bool),

    /// (U32 .. -- Struct): Construct a new Struct
    /// consuming stack values as defined by the U32
    Struct,    

    /// (U32, U32 -- U32): Add two u32 values
    AddU32,
}







