
use super::*;

//mod compile;
//use crate::typed_vm::compile::*;

mod table;
use self::table::{FnTable,TableTypes,CursorTypes};

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
enum Value {
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

    /// (Usize Value(s) -- ): Remove the top Usize values of the stack   
    PopN,

    /// (Value, Value -- Value, Value): Swap the top to value of the stack.
    Swap,

    /// Op::SwapN ( Usize, Usize, ... -- ...): Swaps the top and bott usize slices
    /// of the stack.
    SwapN,

    /// (Value -- Value, Value): Copy the top value of the stack.
    Copy,

    /// (Usize -- Value(s) ): Copy the top usize values of the stack.
    CopyMany,

    /// (usize -- Value): Copy the value from down the stack with the depth
    ///  provided by the usize
    CopyFrom,

    /// (Usize Usize -- Value(s)): Copy values from down the stack with the depth
    ///  provided by the first usize and the umber from the second.
    CopyManyFrom,

    /// ( usize -- Value ): Copy the var from frame offset usize to the top
    /// of the stack.
    Load,

    /// ( usize usize -- Value(sP) ): Copy the var from frame offset usize with a size
    /// specified by the second usize to the top
    /// of the stack.
    LoadN,

    /// ( usize Value --  ): Write usize frame offset with the Value.
    Store,

    /// ( usize usize Value(s) --  ): Write usize starting frame offset with 
    /// the second usize number off the top of the sack.
    StoreN,

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

    /// (Cursor -- Cursor, bool): Returns true when the cursor is
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

    /// (F32, F32 -- F32): Add two f32s.
    AddF32,

    /// (F64, F64 -- F64): Add two f64s
    AddF64,

    /// (U32, U32 -- U32): Add two u32s.
    AddU32,

    /// (U64, U64 -- U64): Add two u64s
    AddU64,

    /// (I32, I32 -- I32): Add two i32s.
    AddI32,

    /// (I64, I64 -- I64): Add two i64s
    AddI64,
}



use std::collections::BTreeMap;
#[derive(Debug)]
pub struct Module {
    pub start: usize,
    pub code: Vec<Op>,
    pub functions: FnTable,
    pub types: BTreeMap<u32,Vec<Type>>,
}

#[derive(Debug)]
struct RetInfo {
    instruction_pointer: usize,
    frame_ptr: usize,
    ret_count: usize,
}
pub struct Vm {
    frame_ptr: usize,
    instruction_pointer: usize,
    stack: Vec<Value>,
    types: BTreeMap<u32,Vec<Type>>,
    call_stack: Vec<RetInfo>,
    code: Vec<Op>,
}

#[derive(Debug)]
pub enum VmError {
    InvalidOperation,
    TypeCheck,
}

impl Vm {
    pub fn new(module: Module) -> Self {
        let table = TableTypes::Fn(module.functions);
        let stack = 
            vec![Value::Table(table)];

        let bottom = RetInfo {
            instruction_pointer: 0,
            frame_ptr: 0,
            ret_count: 0,
        };

         Vm {
            frame_ptr: 1,
            instruction_pointer: module.start,
            stack,
            types: module.types,
            call_stack: vec![bottom],
            code: module.code,
        }
    }


    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn stack<'a>(&'a self) -> &'a Vec<Value> {
        &self.stack
    }

    pub fn code<'a>(&'a self) -> &'a Vec<Op> {
        &self.code
    }

    pub fn stack_get<'a>(&'a self, index: usize) -> Option<&'a Value> {
        self.stack.get(index)
    }

    pub fn run(&mut self) -> Result<(), VmError> {

        let mut halt = false;

        while halt == false {
            halt = self.step()?;
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, VmError> {
        let ptr = self.instruction_pointer;
        self.instruction_pointer += 1;
        match &self.code[ptr] {
            Op::Noop => {},

            Op::Halt => {
                return Ok(true);
            },

            Op::Pop => {
                self.pop()?;
            },

            Op::PopN => {
                let Value::Usize(count) = self.pop()? else {
                    unreachable!();
                };

                let len = self.stack.len() - count;
                self.stack.truncate(len);
            },

            Op::Swap => {

                let last = self.stack.len() - 1;

                let second = last - 1;

                self.stack.swap(last, second);
            },

            Op::SwapN => {
                let Value::Usize(top_size) = self.pop()? else {
                    unreachable!();
                };

                let Value::Usize(bottom_size) = self.pop()? else {
                    unreachable!();
                };

                let at = self.stack.len() - top_size;
                let mut top = self.stack.split_off(at);

                let at = self.stack.len() - bottom_size;
                let mut bottom = self.stack.split_off(at);

                self.stack.append(&mut top);
                self.stack.append(&mut bottom);
            }

            Op::Copy => {
                let value = self.stack.last().unwrap();
                let copy = self.copy_value(value)?;
                self.stack.push(copy);
            },

            Op::CopyMany => {
                let Value::Usize(count) = self.pop()? else {
                    unreachable!();
                };
                
                let start = self.stack.len() - count;
                let end = self.stack.len();

                for index in start..end {
                    let value = self.stack.get(index).unwrap();
                    let copy = self.copy_value(value)?;
                    self.stack.push(copy);
                }
            },

            Op::CopyFrom => {
                let Value::Usize(depth) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let index = self.stack.len() - depth;
                let value = self.stack.get(index).unwrap();
                let copy = self.copy_value(value)?;
                self.stack.push(copy);
            },


            Op::CopyManyFrom => {
                let Value::Usize(depth) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let Value::Usize(count) = self.pop()? else {
                    unreachable!();
                };

                let end = self.stack.len() - depth;
                let start = end - count;
                for index in start..(end + 1) {
                    let value = self.stack.get(index).unwrap();
                    let copy = self.copy_value(value)?;
                    self.stack.push(copy);
                }
            },

            Op::Load => {
                let Value::Usize(offset) = self.pop()? else {
                    unreachable!()
                };

                let index = offset + self.frame_ptr;
                let value = self.stack.get(index).unwrap();
                let copy = self.copy_value(value)?;
                self.stack.push(copy);
            },

            Op::LoadN => {
                let Value::Usize(offset) = self.pop()? else {
                    unreachable!()
                };

                let Value::Usize(count) = self.pop()? else {
                    unreachable!()
                };

                let index = offset + self.frame_ptr;
                for i in 0..count {
                    let value = self.stack.get(index - i).unwrap();
                    let copy = self.copy_value(value)?;
                    self.stack.push(copy);
                }
            },

            
            Op::Store => {
                let Value::Usize(offset) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let to = offset + self.frame_ptr;
                let from = self.stack.len() - 1;
                self.stack.swap(from, to);
               
                self.pop();
            },
            
            Op::StoreN => {
                let Value::Usize(offset) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let Value::Usize(count) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let to = offset + self.frame_ptr;
                let from = self.stack.len() - 1;
                for i in 0..count {
                    self.stack.swap(from - i, to - i);
                }
               
                self.stack.truncate(from - (count -1) );
            },

            Op::Call => {
                let Value::Function{ptr: index} = self.pop()? else {
                    unimplemented!()
                };
                let Value::Usize(arg_count) = self.pop()? else {
                    unimplemented!()
                };
                let Value::Usize(ret_count) = self.pop()? else {
                    unimplemented!()
                };

                let ret = RetInfo {
                    instruction_pointer: self.instruction_pointer,
                    frame_ptr: self.frame_ptr,
                    ret_count: ret_count,
                };
        
                
                self.frame_ptr = self.stack.len() - arg_count ;
                self.instruction_pointer = index;
                self.call_stack.push(ret);
            },

            Op::Return => {
                let ret = self.call_stack.pop().unwrap();
                //assert!(self.frame_ptr + ret.ret_count == (self.stack.len() - 1));
                self.instruction_pointer = ret.instruction_pointer;
                self.frame_ptr = ret.frame_ptr;            }

            Op::Table => {
                todo!();
            },

            Op::Query => {
                let Some(Value::Struct {field_count}) = self.stack.pop() else {
                    unimplemented!()
                };

                let at = self.stack.len() - field_count;
                let mut fields = self.stack.split_off(at);

                let Some(Value::Table(table)) = self.stack.pop() else {
                    unimplemented!()
                };

                let cursor = table.find(&mut fields);

                self.stack.push(Value::Cursor(cursor));                
            }

            Op::Found => {
                todo!()
            }

            Op::Read => {
                let Some(Value::Cursor(cursor)) = self.stack.pop() else {
                    unimplemented!()
                };

                cursor.read(&mut self.stack)?;
                self.stack.push(Value::Cursor(cursor));
            },

            Op::Insert => {
                todo!();
            },

            Op::Update => {
                todo!();
            },

            Op::Delete => {
                todo!()
            },

            Op::Advance => {
                todo!();
            },

            Op::Close => {
                todo!();
            }

            Op::None => {
                self.stack.push(Value::None);
            },

            Op::Fn(v) => {
                self.stack.push(Value::Function { ptr: *v });
            },

            Op::F32(value) => {
                self.stack.push(Value::F32(*value));
            },

            Op::F64(value) => {
                self.stack.push(Value::F64(*value));
            },
            Op::I32(value) => {
                self.stack.push(Value::I32(*value));
            },

            Op::I64(value) => {
                self.stack.push(Value::I64(*value));
            },
            Op::U32(value) => {
                self.stack.push(Value::U32(*value));
            },

            Op::U64(value) => {
                self.stack.push(Value::U64(*value));
            },

            Op::Usize(value) => {
                self.stack.push(Value::Usize(*value));
            },

            Op::Bool(value) => {
                self.stack.push(Value::Bool(*value));
            },

            Op::Struct => {
                let Value::Usize(field_count) = self.stack.pop().unwrap() else {
                    unreachable!();
                };

                self.stack.push(Value::Struct{field_count});
            },

            Op::AddF32 => {
                let Value::F32(a) = self.pop()? else {
                    unreachable!()
                };
                let Value::F32(b) = self.pop()? else {
                    unreachable!()
                };
                self.stack.push(Value::F32(a+b));
            },

            Op::AddF64 => {
                let Value::F64(a) = self.pop()? else {
                    unreachable!()
                };
                let Value::F64(b) = self.pop()? else {
                    unreachable!()
                };
                self.stack.push(Value::F64(a+b));
            },

            
            Op::AddU32 => {
                let Value::U32(a) = self.pop()? else {
                    unreachable!()
                };
                let Value::U32(b) = self.pop()? else {
                    unreachable!()
                };
                self.stack.push(Value::U32(a+b));
            },

            Op::AddU64 => {
                let Value::U64(a) = self.pop()? else {
                    unreachable!()
                };
                let Value::U64(b) = self.pop()? else {
                    unreachable!()
                };
                self.stack.push(Value::U64(a+b));
            },
            
            Op::AddI32 => {
                let Value::I32(a) = self.pop()? else {
                    unreachable!()
                };
                let Value::I32(b) = self.pop()? else {
                    unreachable!()
                };
                self.stack.push(Value::I32(a+b));
            },

            Op::AddI64 => {
                let Value::I64(a) = self.pop()? else {
                    unreachable!()
                };
                let Value::I64(b) = self.pop()? else {
                    unreachable!()
                };
                self.stack.push(Value::I64(a+b));
            },
        }
        Ok(false)
    }
    
    fn copy_value( &self, value: &Value) -> Result<Value, VmError> {
        let result = match value {
            Value::None => Value::None,
            Value::F32(v) => Value::F32(*v),
            Value::F64(v) => Value::F64(*v),
            Value::I32(v) => Value::I32(*v),
            Value::I64(v) => Value::I64(*v),
            Value::U32(v) => Value::U32(*v),
            Value::U64(v) => Value::U64(*v),
            Value::Usize(v) => Value::Usize(*v),
            Value::StringRef{index: _} => {
                return Err(VmError::InvalidOperation)
            },
            Value::Bool(v) => Value::Bool(*v),
            Value::Struct {field_count} => Value::Struct { field_count: *field_count }, 
            Value::Table (_) => {
                return Err(VmError::InvalidOperation)
            },
            Value::Cursor(_) => {
                return Err(VmError::InvalidOperation)
            },
            Value::Function {ptr} => Value::Function { ptr:*ptr },
        };
        Ok(result)
    }

    fn eq_value(a: &Value, b: &Value) -> Result<bool, VmError> {
        let result = match (a, b) {
            (Value::None, Value::None) => false,
            (Value::F32(x), Value::F32(y)) => *x == *y,
            (Value::F64(x), Value::F64(y)) => *x == *y,
            (Value::I32(x), Value::I32(y)) => *x == *y,
            (Value::I64(x), Value::I64(y)) => *x == *y,
            (Value::U32(x), Value::U32(y)) => *x == *y,
            (Value::U64(x), Value::U64(y)) => *x == *y,
            (Value::Usize(x), Value::Usize(y)) => *x == *y,
            (Value::StringRef{index: x}, Value::StringRef{index: y}) => *x == *y,
            (Value::Bool(x), Value::Bool(y)) => *x == *y,
            (Value::Struct {field_count: x}, Value::Struct {field_count: y}) => 
                *x == *y, 
            (Value::Table (_), Value::Table (_)) => return Err(VmError::InvalidOperation),
            (Value::Cursor(_), Value::Cursor(_)) => return Err(VmError::InvalidOperation),
            (Value::Function {ptr: x}, Value::Function {ptr: y}) => 
                *x == *y,
            _ => {
                return Err(VmError::InvalidOperation);
            },
        };
        Ok(result)
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        Ok(self.stack.pop().unwrap())
    }


}

#[cfg(test)]
mod test;