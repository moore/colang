
use std::collections::BTreeMap;

use crate::dyn_vm::VmError;

use super::*;

pub trait Table<T,E> {
    type Cursor: Cursor<T,E,Table=Self>;
    fn find(self, query: &mut Vec<T>) -> Self::Cursor;
}

pub trait Cursor<T,E> {
    type Table: Table<T,E>;

    /// Returns true if the cursor is at a record that
    ///  matches the query for this cursor.
    fn found(&self) -> bool;

    /// Pushes a struct on to the stack matching the record at the cursor.
    /// If no record exists at the cursor, None is pushed on to the stack.
    fn read(&self, stack: &mut Vec<T>) -> Result<(), E>;

    /// Consumes a struct from the stack which matches the record type
    /// and adds it to the table in the position fallowing the cursor.
    /// The resulting cursor is advanced to point at the inserted record.
    fn insert(&mut self, stack: &mut Vec<T>) -> Result<(), E>;

    /// Consumes a struct from the stack which matches the record type
    /// and replaces the record in the table in the position of the cursor.
    fn update(&mut self, stack: &mut Vec<T>) -> Result<(), E>;

    /// Deletes the record at the cursor and advances the cursor to the
    /// next matching record or the end of the table.
    fn delete(&mut self) -> Result<(), E>;

    /// advances the cursor to the next matching record or the end of the 
    /// table.
    fn advance(&mut self) -> Result<bool, E>;

    /// Closes the cursor and returns the underlying table.
    fn close(self) -> Self::Table;
}



