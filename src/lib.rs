
use std::collections::BTreeMap;

mod table;
use crate::table::*;

#[derive(Debug, Clone)]
pub enum Type {
    None,
    U32,
    StringRef,
    Bool,
    Struct(u32),
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
        field_count: u32,
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




pub struct Module {
    start: usize,
    code: Vec<Op>,
    functions: FnTable,
    types: BTreeMap<u32,Vec<Type>>,
}



type Idx = usize;

struct RetInfo {
    instruction_pointer: usize,
    frame_ptr: Idx,
    ret_count: usize,
}
pub struct Vm {
    frame_ptr: Idx,
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

         Vm {
            frame_ptr: module.start,
            instruction_pointer: module.start,
            stack,
            types: module.types,
            call_stack: Vec::new(),
            code: module.code,
        }
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
                let value = self.pop()?;

                if let Value::Struct { field_count } = value {
                    let len = self.stack.len() - field_count as usize;
                    self.stack.truncate(len);
                }

                self.inc_op();
            },

            Op::Swap => {

                let last = self.stack.len() - 1;

                let last_len = match self.stack.get(last) {
                    Some(Value::Struct { field_count}) => *field_count + 1,
                    _ => 1,
                };

                let second = last - last_len as usize;

                let second_len = match self.stack.get(second) {
                    Some(Value::Struct { field_count}) => *field_count + 1,
                    _ => 1,
                };

                if last_len == 1 && second_len == 1 {
                    self.stack.swap(last, second);
                } else {
                    let at = self.stack.len() - last_len as usize;
                    let mut top = self.stack.split_off(at);

                    let at = self.stack.len() - second_len as usize;
                    let mut bottom = self.stack.split_off(at);

                    dbg!(&top);
                    dbg!(&bottom);
                    self.stack.append(&mut top);
                    self.stack.append(&mut bottom);
                }
                self.inc_op();
            },

            Op::Copy=> {
                let index = self.stack.len() - 1;
                self.copy(index)?;
                self.inc_op();
            },

            Op::CopyFrom => {
                let Value::Usize(depth) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let index = self.stack.len() - depth;
                self.copy(index)?;
                self.inc_op();
            },

            Op::Call => {
                dbg!(&self.stack);
                let Value::Function{ptr: index} = self.pop()? else {
                    unimplemented!()
                };
                let Value::U32(arg_count) = self.pop()? else {
                    unimplemented!()
                };
                let Value::U32(ret_count) = self.pop()? else {
                    unimplemented!()
                };
                self.call(index, arg_count as usize, ret_count as usize);
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

                let at = self.stack.len() - field_count as usize;
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

            Op::U32(value) => {
                self.stack.push(Value::U32(*value));
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
                let Value::U32(field_count) = self.stack.pop().unwrap() else {
                    unreachable!();
                };

                self.stack.push(Value::Struct{field_count});
                self.inc_op();
            },

            Op::AddU32 => {
                let Value::U32(first) = self.stack.pop().unwrap() else {
                    unreachable!();
                };
                let Value::U32(second) = self.stack.pop().unwrap() else {
                    unreachable!();
                };

                self.stack.push(Value::U32(first + second));
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
            let start = index - *field_count as usize;
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

    fn copy_value( &self, value: &Value) -> Result<Value, VmError> {
        let result = match value {
            Value::None => Value::None,
            Value::U32(v) => Value::U32(*v),
            Value::Usize(v) => Value::Usize(*v),
            Value::StringRef{index: _} => return Err(VmError::InvalidOperation),
            Value::Bool(v) => Value::Bool(*v),
            Value::Struct {field_count} => Value::Struct { field_count: *field_count }, 
            Value::Table (_) => return Err(VmError::InvalidOperation),
            Value::Cursor(_) => return Err(VmError::InvalidOperation),
            Value::Function {ptr} => Value::Function { ptr:*ptr },
        };
        Ok(result)
    }

    fn inc_op(&mut self) {
        self.instruction_pointer += 1;
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        Ok(self.stack.pop().unwrap())
    }

    fn call(&mut self, index: usize, arg_count: usize, ret_count: usize) {
        let ret = RetInfo {
            instruction_pointer: self.instruction_pointer + 1,
            frame_ptr: self.frame_ptr,
            ret_count: ret_count,
        };

        self.frame_ptr = self.stack.len() - arg_count - 1;
        self.instruction_pointer = index;
        self.call_stack.push(ret);
    }

    fn ret(&mut self) {
        let ret = self.call_stack.pop().unwrap();
        assert!(self.frame_ptr + ret.ret_count == (self.stack.len() - 1));
        self.instruction_pointer = ret.instruction_pointer;
        self.frame_ptr = ret.frame_ptr;
    }
}



#[cfg(test)]
mod test;

