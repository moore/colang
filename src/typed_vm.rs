
use super::*;

mod compile;
use crate::typed_vm::compile::*;

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
        match &self.code[ptr] {
            Op::Noop => {
                self.inc_op();
            },

            Op::Halt => {
                return Ok(true);
            },

            Op::Pop => {
                self.pop_value()?;

                self.inc_op();
            },

            Op::Swap => {

                let last = self.stack.len() - 1;

                let last_len = match self.stack.get(last) {
                    Some(Value::Struct { field_count}) => *field_count + 1,
                    _ => 1,
                };

                let second = last - last_len;

                let second_len = match self.stack.get(second) {
                    Some(Value::Struct { field_count}) => *field_count + 1,
                    _ => 1,
                };

                if last_len == 1 && second_len == 1 {
                    self.stack.swap(last, second);
                } else {
                    let at = self.stack.len() - last_len;
                    let mut top = self.stack.split_off(at);

                    let at = self.stack.len() - second_len;
                    let mut bottom = self.stack.split_off(at);

                    self.stack.append(&mut top);
                    self.stack.append(&mut bottom);
                }
                self.inc_op();
            },

            Op::Copy => {
                let index = self.stack.len() - 1;
                self.copy(index)?;
                self.inc_op();
            },

            Op::CopyFrom => {
                let Value::Usize(depth) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };
                dbg!(&self.code);
                dbg!(&self.stack);
                dbg!(depth);
                dbg!(self.stack.len());

                let index = self.stack.len() - depth;
                self.copy(index)?;
                self.inc_op();
            },

            Op::Load => {
                let Value::Usize(offset) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let index = offset + self.frame_ptr;
                self.copy(index)?;
                self.inc_op();
            },

            
            Op::Store => {
                let Value::Usize(offset) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let to = offset + self.frame_ptr;
                let from = self.stack.len() - 1;
                self.store(to, from)?;
                self.pop_value()?;
                self.inc_op();
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
                self.call(index, arg_count, ret_count);
            },

            Op::Return => {
                self.ret();
            }

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

                self.inc_op();
                
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
                self.inc_op();
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
                self.inc_op();
            },

            Op::Fn(v) => {
                self.stack.push(Value::Function { ptr: *v });
                self.inc_op();
            },

            Op::F32(value) => {
                self.stack.push(Value::F32(*value));
                self.inc_op();
            },

            Op::F64(value) => {
                self.stack.push(Value::F64(*value));
                self.inc_op();
            },
            Op::I32(value) => {
                self.stack.push(Value::I32(*value));
                self.inc_op();
            },

            Op::I64(value) => {
                self.stack.push(Value::I64(*value));
                self.inc_op();
            },
            Op::U32(value) => {
                self.stack.push(Value::U32(*value));
                self.inc_op();
            },

            Op::U64(value) => {
                self.stack.push(Value::U64(*value));
                self.inc_op();
            },

            Op::Usize(value) => {
                self.stack.push(Value::Usize(*value));
                self.inc_op();
            },

            Op::Bool(value) => {
                self.stack.push(Value::Bool(*value));
                self.inc_op();
            },

            Op::Struct => {
                let Value::Usize(field_count) = self.stack.pop().unwrap() else {
                    unreachable!();
                };

                self.stack.push(Value::Struct{field_count});
                self.inc_op();
            },

            Op::Add => {
                let first = self.stack.pop().unwrap();
                let second = self.stack.pop().unwrap();
                use Value::*;

                let sum = match (first, second) {
                    (F32(a), F32(b)) => F32(a+ b),
                    (F64(a), F64(b)) => F64(a+ b),

                    (I32(a), I32(b)) => I32(a+ b),
                    (I64(a), I64(b)) => I64(a+ b),

                    (U32(a), U32(b)) => U32(a+ b),
                    (U64(a), U64(b)) => U64(a+ b),
                    _ => return Err(VmError::TypeCheck),
                };

                self.stack.push(sum);
                self.inc_op();
            },
        }
        Ok(false)
    }

    fn copy(&mut self, index: usize) -> Result<(), VmError> {
        let value = self.stack.get(index).unwrap();

        if let Value::Struct {field_count} = value {
            // We don't have to add one to index because
            // field count dose not include the struct value itself
            let start = index - *field_count;
            let end = index + 1;
            for i in start..end {
                let dup = self.copy_value(&self.stack[i])?;
                self.stack.push(dup);
            }
        } else {
            let copy = self.copy_value(value)?;
            self.stack.push(copy);
        }
        Ok(())
    }

    fn store(&mut self, to: usize, from: usize) -> Result<(), VmError> {
        let target = self.stack.get(to).unwrap();
        let source = self.stack.get(from).unwrap();

        if !matches![Value::None, target] {
            if ! Vm::eq_value(source, target)? {
                return Err(VmError::TypeCheck);
            }
        }   

        if let Value::Struct {field_count} = target {
            let to_start = to - *field_count;
            let from_start = from - *field_count;
            for i in 0..*field_count {
                let to_next = to_start + i;
                let from_next = from_start +1;
                self.store(to_next, from_next)?;
            }
        } else {
            self.stack.swap(to, from);
        }
        Ok(())
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

    fn inc_op(&mut self) {
        self.instruction_pointer += 1;
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        Ok(self.stack.pop().unwrap())
    }

    fn pop_value(&mut self) -> Result<(), VmError> {
        let value = self.pop()?;

        if let Value::Struct { field_count } = value {
            let len = self.stack.len() - field_count;
            self.stack.truncate(len);
        }

        Ok(())
    }

    fn call(&mut self, index: usize, arg_count: usize, ret_count: usize) {
        let ret = RetInfo {
            instruction_pointer: self.instruction_pointer + 1,
            frame_ptr: self.frame_ptr,
            ret_count: ret_count,
        };

        
        self.frame_ptr = self.stack.len() - arg_count ;
        self.instruction_pointer = index;
        self.call_stack.push(ret);
    }

    fn ret(&mut self) {
        let ret = self.call_stack.pop().unwrap();
        //assert!(self.frame_ptr + ret.ret_count == (self.stack.len() - 1));
        self.instruction_pointer = ret.instruction_pointer;
        self.frame_ptr = ret.frame_ptr;
    }
}

#[cfg(test)]
mod test;