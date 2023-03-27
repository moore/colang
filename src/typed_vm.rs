use super::*;

use std::collections::BTreeMap;
#[derive(Debug)]
pub struct Module {
    pub start: usize,
    pub code: Vec<Op>,
    pub functions: FnTable,
    pub types: BTreeMap<u32,Vec<Type>>,
}

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

         Vm {
            frame_ptr: module.start,
            instruction_pointer: module.start,
            stack,
            types: module.types,
            call_stack: Vec::new(),
            code: module.code,
        }
    }


    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn stack<'a>(&'a self) -> &'a Vec<Value> {
        &self.stack
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
                let value = self.pop()?;

                if let Value::Struct { field_count } = value {
                    let len = self.stack.len() - field_count;
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
            Value::StringRef{index: _} => return Err(VmError::InvalidOperation),
            Value::Bool(v) => Value::Bool(*v),
            Value::Struct {field_count} => Value::Struct { field_count: *field_count }, 
            Value::Table (_) => return Err(VmError::InvalidOperation),
            Value::Cursor(_) => return Err(VmError::InvalidOperation),
            Value::Function {ptr} => Value::Function { ptr:*ptr },
        };
        Ok(result)
    }

    fn eq_value(a: &Value, b: &Value) -> Result<bool, VmError> {
        let result = match (a, b) {
            (Value::None, Value::None) => false,
            (Value::U32(x), Value::U32(y)) => *x == *y,
            (Value::Usize(x), Value::Usize(y)) => *x == *y,
            (Value::StringRef{index: x}, Value::StringRef{index: y}) => *x == *y,
            (Value::Bool(x), Value::Bool(y)) => *x == *y,
            (Value::Struct {field_count: x}, Value::Struct {field_count: y}) => 
                *x == *y, 
            (Value::Table (_), Value::Table (_)) => return Err(VmError::InvalidOperation),
            (Value::Cursor(_), Value::Cursor(_)) => return Err(VmError::InvalidOperation),
            (Value::Function {ptr: x}, Value::Function {ptr: y}) => 
                *x == *y,
            _ => return Err(VmError::InvalidOperation),
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