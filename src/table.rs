
use std::ops::Index;

use super::*;

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
                if query.len() != 2 {
                    unimplemented!()
                }

                let Some(key) = query.get(1) else {
                    unreachable!()
                };

                if !matches!(key, Value::U32(_)) {
                    unimplemented!()
                }
                
                let Some(value) = query.get(0) else {
                    unreachable!()
                };

                if !matches!(value, Value::None) {
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
    fn insert(&mut self, _stack: &mut Vec<Value>) -> Result<(), VmError>{
        Err(VmError::InvalidOperation)
    }

    /// Consumes a struct from the stack which matches the record type
    /// and replaces the record in the table in the position of the cursor.
    fn update(&mut self, _stack: &mut Vec<Value>) -> Result<(), VmError> {
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
    pub fn new() -> Self {
        FnTable {
            functions: BTreeMap::new(),
        }
    }

    pub fn add_fn(&mut self, index: u32, ptr: usize) {
        self.functions.insert(index, ptr);
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

