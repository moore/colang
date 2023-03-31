use super::*;

mod compile;
use crate::dyn_vm::compile::*;

use std::collections::BTreeMap;


#[derive(Debug, Clone)]
enum Type {
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
    Symbol(String),
    StringRef{
        index: usize,
    },
    Bool(bool),
    Struct(Vec<Value>), 
    Function(usize),
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

    /// ( Symbol -- Value ): Copy the var from frame.
    Load,


    /// ( Symbol Value -- ): Write the to the value to the variable
    /// which matches the Symbol based on the current scope.
    Store,

    /// ( Symbol -- Function ): Pushes the function named by the symbol on
    /// to the stack.
    GetFn,


    /// (Symbol -- ): Call the given function name. 
    Call,

    /// ( -- )
    Return,

    /// ( -- None): Push None on to the stack.
    None,

    /// ( -- Symbol ): Push a Symbol on the stack,
    Symbol(String),

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

    /// (Usize, Struct -- Value Struct): Copy a value out of a struct indexed by
    /// Usize.
    StructRead,

    /// (Number<T>, Number<T> --Number<T>): Add two numbers of a matching type
    /// and put the result with the same type on the stack.
    Add,
}


#[derive(Debug)]
pub struct Module {
    pub start: usize,
    pub code: Vec<Op>,
    pub functions: BTreeMap<String,usize>,
}

#[derive(Debug)]
struct Scope {
    vars: BTreeMap<String, Value>,
}

impl Scope {
    fn new() -> Self {
        Scope { vars: BTreeMap::new() }
    }
}

#[derive(Debug)]
pub struct Vm {
    frame_ptr: usize,
    instruction_pointer: usize,
    stack: Vec<Value>,
    call_stack: Vec<usize>,
    scope_stack: Vec<Scope>,
    functions:BTreeMap<String,usize>,
    code: Vec<Op>,
}

#[derive(Debug)]
pub enum VmError {
    InvalidOperation,
    TypeCheck,
    UnknownVar(String),
    UnknownFunction(String),
}

impl Vm {
    pub fn new(module: Module) -> Self {
        let stack = vec![];

        let bottom = 0;

        let global_scope = Scope {
            vars: BTreeMap::new(),
        };


         Vm {
            frame_ptr: 1,
            instruction_pointer: module.start,
            stack,
            call_stack: vec![bottom],
            scope_stack: vec![global_scope],
            functions: module.functions,
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
                let second = last - 1;

                self.stack.swap(last, second);
                
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

                let index = self.stack.len() - depth;
                self.copy(index)?;
                self.inc_op();
            },

            Op::Load => {
                let Value::Symbol(name) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let scope = self.scope_stack.last().unwrap();


                let Some(value) = scope.vars.get(&name) else {
                    return Err(VmError::UnknownVar(name));
                };

                self.stack.push(self.copy_value(value)?);

                self.inc_op();
            },

            
            Op::Store => {
                let Value::Symbol(name) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };
                
                let value = self.pop()?;
                let scope = self.scope_stack.last_mut().unwrap();

                scope.vars.insert(name, value);

                self.inc_op();
            },
            
            Op::GetFn => {
                let Value::Symbol(name) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let Some(ptr) = self.functions.get(&name) else {
                    return Err(VmError::UnknownFunction(name));
                };

                self.stack.push(Value::Function(*ptr));
                self.inc_op();
            },


            Op::Call => {
                let Value::Function(ptr) = self.pop()? else {
                    unimplemented!()
                };

                let mut scope = Scope::new();

                self.scope_stack.push(scope);
                self.call_stack.push(self.instruction_pointer + 1);

                self.instruction_pointer = ptr;
            },

            Op::Return => {
                self.scope_stack.pop();
                self.instruction_pointer = self.call_stack.pop().unwrap();
            }

            
            Op::None => {
                self.stack.push(Value::None);
                self.inc_op();
            },

            Op::Symbol(v) => {
                self.stack.push(Value::Symbol(v.clone()));
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
                let at = self.stack_len() - field_count;
                let mut values = self.stack.split_off(at);

                values.reverse();

                self.stack.push(Value::Struct(values));
                self.inc_op();
            },

            Op::StructRead => {
                let Value::Usize(index) = self.stack.pop().unwrap() else {
                    return Err(VmError::TypeCheck);
                };

                let Some(Value::Struct(args)) = self.stack.last() else {
                    return Err(VmError::TypeCheck);
                };

                let Some(value) = args.get(index) else {
                    return Err(VmError::TypeCheck);
                };

                let new = self.copy_value(value)?;
                self.stack.push(new);
                self.inc_op();
            }

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

    
        let copy = self.copy_value(value)?;
        self.stack.push(copy);

        Ok(())
    }

    fn store(&mut self, to: usize, from: usize) -> Result<(), VmError> {
        let Value::Symbol(name) = self.pop()? else {
            return Err(VmError::TypeCheck);
        };

        let value = self.pop()?;
        self.scope_stack.last_mut().unwrap().vars.insert(name, value);

        Ok(())
    }


    

    fn copy_value( &self, value: &Value) -> Result<Value, VmError> {
        let result = match value {
            Value::None => Value::None,
            Value::Symbol(v) => Value::Symbol(v.clone()),
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
            Value::Struct(v) => {
                let mut values = Vec::new();
                for value in v {
                    values.push(self.copy_value(value)?);
                }
                
                Value::Struct(values)
            }, 
           
            Value::Function(ptr) => Value::Function(*ptr),
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
            (Value::Struct {..}, Value::Struct {..}) => 
                return Err(VmError::InvalidOperation),
            (Value::Function {..}, Value::Function {..}) => 
                return Err(VmError::InvalidOperation),
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
        Ok(())
    }
}

#[cfg(test)]
mod test;