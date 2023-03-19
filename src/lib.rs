
use std::collections::{BTreeMap, btree_map::Values};

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

    /// (Function, U32, U32 -- ): Call the given function ref, it is 
    /// expect to consume the number of args given by the first u32 
    /// and return the number specified byt the second. 
    Call,

    /// ( -- )
    Return,

    /// (u32 -- Table) Construct a table with given type index.
    Table,

    /// (Table, Struct -- Cursor) Querying a table using the Struct
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

    /// ( -- U32): Push a U32 on to the stack.
    U32(u32),

    /// ( -- Value::Bool): Push a Value::Bool on to the stack
    Bool(bool),

    /// (StructType .. -- Struct): Construct a new Struct
    /// consuming stack values as defined by the StructType
    Struct,    

    /// (U32, U32 -- U32): Add two u32 values
    AddU32,
}


pub trait Table {
    type Cursor: Cursor<Table=Self>;
    fn find(self, query: &mut Vec<Value>) -> Self::Cursor;
}

pub trait Cursor {
    type Table: Table;

    /// Returns true if the cursor is at a record that
    ///  matches the query for this cursor.
    fn found(&self) -> bool;

    /// Pushes a struct on to the stack matching the record at the cursor.
    /// If no record exists at the cursor, None is pushed on to the stack.
    fn read(&self, stack: &mut Vec<Value>) -> Result<(), VmError>;

    /// Consumes a struct from the stack which matches the record type
    /// and adds it to the table in the position fallowing the cursor.
    /// The resulting cursor is advanced to point at the inserted record.
    fn insert(&mut self, stack: &mut Vec<Value>) -> Result<(), VmError>;

    /// Consumes a struct from the stack which matches the record type
    /// and replaces the record in the table in the position of the cursor.
    fn update(&mut self, stack: &mut Vec<Value>) -> Result<(), VmError>;

    /// Deletes the record at the cursor and advances the cursor to the
    /// next matching record or the end of the table.
    fn delete(&mut self) -> Result<(), VmError>;

    /// advances the cursor to the next matching record or the end of the 
    /// table.
    fn advance(&mut self) -> Result<bool, VmError>;

    /// Closes the cursor and returns the underlying table.
    fn close(self) -> Self::Table;
}



#[derive(Debug)]
pub enum TableTypes {
    Fn(FnTable),
}

impl Table for TableTypes {
    type Cursor = CursorTypes;

    fn find(self, query: &mut Vec<Value>) -> Self::Cursor {
        match self {
            TableTypes::Fn(table) => {
                // BUG: this type checking should be based on Vm.types
                if query.len() != 1 {
                    unimplemented!()
                }

                let Some(value) = query.get(0) else {
                    unreachable!()
                };

                if !matches!(value, Value::U32(_)) {
                    unimplemented!()
                }
                

                CursorTypes::Fn(table.find(query))
            },
        }
    }
}

#[derive(Debug)]
pub enum CursorTypes {
    Fn(FnCursor),
}

impl Cursor for CursorTypes {
    type Table = TableTypes;
    fn found(&self) -> bool {
        todo!()
    }

    /// Pushes a struct on to the stack matching the record at the cursor.
    /// If no record exists at the cursor, None is pushed on to the stack.
    fn read(&self, stack: &mut Vec<Value>) -> Result<(), VmError> {
        match self {
            CursorTypes::Fn(table) => table.read(stack),
        }
    }

    /// Consumes a struct from the stack which matches the record type
    /// and adds it to the table in the position fallowing the cursor.
    /// The resulting cursor is advanced to point at the inserted record.
    fn insert(&mut self, stack: &mut Vec<Value>) -> Result<(), VmError>{
        match self {
            CursorTypes::Fn(table) => table.insert(stack),
        }
    }

    /// Consumes a struct from the stack which matches the record type
    /// and replaces the record in the table in the position of the cursor.
    fn update(&mut self, stack: &mut Vec<Value>) -> Result<(), VmError> {
        match self {
            CursorTypes::Fn(table) => table.update(stack),
        }
    }

    /// Deletes the record at the cursor and advances the cursor to the
    /// next matching record or the end of the table.
    fn delete(&mut self) -> Result<(), VmError> {
        match self {
            CursorTypes::Fn(table) => table.delete(),
        }
     }

    /// advances the cursor to the next matching record or the end of the 
    /// table.
    fn advance(&mut self) -> Result<bool, VmError> {
        match self {
            CursorTypes::Fn(table) => table.advance(),
        }
    }

    /// Closes the cursor and returns the underlying table.
    fn close(self) -> Self::Table {
        match self {
            CursorTypes::Fn(table) => TableTypes::Fn(table.close()),
        }
    }
}


#[derive(Debug)]
pub struct FnCursor {
    table: FnTable,
    index: usize,
}

impl Cursor for FnCursor {
    type Table = FnTable;
    fn found(&self) -> bool {
        todo!()
    }

    /// Pushes a struct on to the stack matching the record at the cursor.
    /// If no record exists at the cursor, None is pushed on to the stack.
    fn read(&self, stack: &mut Vec<Value>) -> Result<(), VmError> {
        stack.push(Value::Function { ptr: self.index });
        Ok(())
    }

    /// Consumes a struct from the stack which matches the record type
    /// and adds it to the table in the position fallowing the cursor.
    /// The resulting cursor is advanced to point at the inserted record.
    fn insert(&mut self, stack: &mut Vec<Value>) -> Result<(), VmError>{
        Err(VmError::InvalidOperation)
    }

    /// Consumes a struct from the stack which matches the record type
    /// and replaces the record in the table in the position of the cursor.
    fn update(&mut self, stack: &mut Vec<Value>) -> Result<(), VmError> {
        Err(VmError::InvalidOperation)
    }

    /// Deletes the record at the cursor and advances the cursor to the
    /// next matching record or the end of the table.
    fn delete(&mut self) -> Result<(), VmError> {
        Err(VmError::InvalidOperation)
    }

    /// advances the cursor to the next matching record or the end of the 
    /// table.
    fn advance(&mut self) -> Result<bool, VmError> {
        Err(VmError::InvalidOperation)
    }

    /// Closes the cursor and returns the underlying table.
    fn close(self) -> Self::Table {
        return self.table;
    }
}

#[derive(Debug)]
pub struct FnTable {
    functions: BTreeMap<u32,usize>,
}

impl FnTable {
    fn new() -> Self {
        FnTable {
            functions: BTreeMap::new(),
        }
    }
}

impl Table for FnTable {
    type Cursor = FnCursor;
    fn find(self, query: &mut Vec<Value>) -> Self::Cursor {
        let Some(Value::U32(fn_index)) = query.pop() else {
            unreachable!()
        };

        let Some(ptr) = self.functions.get(&fn_index) else {
            unreachable!()
        };

        let ptr = *ptr;
        FnCursor {
            table: self,
            index: ptr, // Bug: How do I know this won't overflow?
        }
    }
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
    InvalidOperation
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
                self.pop()?;
                self.inc_op();
            },

            Op::Swap => {
                let last = self.stack.len() - 1;
                self.stack.swap(last, last - 1);
                self.inc_op();
            },

            Op::Copy=> {
                self.copy()?;
                self.inc_op();
            },

            Op::Call => {
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
                todo!()
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

            Op::U32(value) => {
                self.stack.push(Value::U32(*value));
                self.inc_op();
            },


            Op::Bool(value) => {
                self.stack.push(Value::Bool(*value));
                self.inc_op();
            },

            Op::Struct => {
                todo!();
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

    fn copy(&mut self) -> Result<(), VmError> {
        let value = self.stack.last().unwrap();
        if let Value::Struct {field_count} = value {
            // need to copy all the fields
            todo!()
        }

        let copy = match value {

            Value::None => Value::None,
            Value::U32(v) => Value::U32(*v),
            Value::StringRef{index} => return Err(VmError::InvalidOperation),
            Value::Bool(v) => Value::Bool(*v),
            Value::Struct {field_count} => {
                unreachable!()
            }, 
            Value::Table (_) => return Err(VmError::InvalidOperation),
            Value::Cursor(CursorTypes) => return Err(VmError::InvalidOperation),
            Value::Function {ptr} => Value::Function { ptr:*ptr },
        };
        self.stack.push(copy);
        Ok(())
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

        self.frame_ptr = self.stack.len() - (arg_count + 1);
        self.instruction_pointer = index;
        self.call_stack.push(ret);
    }

    fn ret(&mut self) {
        let ret = self.call_stack.pop().unwrap();
        assert!(self.frame_ptr == (self.stack.len() -1 + ret.ret_count));
        self.instruction_pointer = ret.instruction_pointer;
        self.frame_ptr = ret.frame_ptr;
    }
}



#[cfg(test)]
mod test;

