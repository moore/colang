use super::*;

mod compile;
use crate::dyn_vm::compile::*;

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct VarValue {
    name: String,
    index: usize,
    var_type: Type,
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    name: String,
    offset: usize,
    args: usize,
    vars: Vec<VarValue>,
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
    Symbol(String),
    StringRef{
        index: usize,
    },
    Bool(bool),
    Struct(Vec<Value>), 
    Function(FunctionValue),
}

impl From<Value> for Type {
    fn from(value: Value) -> Self {
        (&value).into()
    }
}

impl From<&Value> for Type {
    fn from(value: &Value) -> Self {
        match value {
            Value::None => Type::None,
            Value::Usize(_) => Type::Usize,
            Value::F32(_) => Type::F32,
            Value::F64(_) => Type::F64,
            Value::U32(_) => Type::U32,
            Value::U64(_) => Type::U64,
            Value::I32(_) => Type::I32,
            Value::I64(_) => Type::I64,
            Value::Symbol(_) => Type::Symbol,
            Value::StringRef{..} => Type::StringRef,
            Value::Bool(_) => Type::Bool,
            Value::Struct(fields)  => {
                let new_fields = fields.iter()
                    .map(|v| v.into()).collect();
                Type::Struct(new_fields)
            },
            Value::Function(from) => {
                let function = Function {
                    name: from.name.clone(),
                    args: vec![Type::Unknown; from.args],
                    vars: from.vars
                        .iter()
                        .map(|v| 
                            Var {
                                name: v.name.clone(), 
                                var_type: v.var_type.clone()
                            })
                        .collect(),
                };
                Type::Function(function)
            },  
        }
    }
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


    /// ( -- ): Pop the address of the top of the call stack and jump
    /// to the popped address.
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
    pub functions: BTreeMap<String,FunctionValue>,
}

#[derive(Debug)]
struct CallStackEntry {
    instruction: usize,
    scope: BTreeMap<usize, Value>
}


#[derive(Debug)]
pub struct Vm {
    instruction_pointer: usize,
    frame: BTreeMap<usize, Value>,
    stack: Vec<Value>,
    call_stack: Vec<CallStackEntry>,
    functions:BTreeMap<String,FunctionValue>,
    code: Vec<Op>,
}

#[derive(Debug)]
pub enum VmError {
    InvalidOperation,
    TypeCheck,
    UnknownVar(usize),
    UnknownFunction(String),
}

impl Vm {
    pub fn new(module: Module) -> Self {
        let stack = vec![];

        let bottom = CallStackEntry { 
            instruction: 0, 
            scope: BTreeMap::new(),
        };


         Vm {
            instruction_pointer: module.start,
            stack,
            frame: BTreeMap::new(),
            call_stack: vec![bottom],
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
        dbg!(&self.stack);
        dbg!(&self.code[ptr]);
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
                let Value::Usize(index) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let Some(value) = self.frame.get(&index) else {
                    return Err(VmError::UnknownVar(index));
                };

                self.stack.push(self.copy_value(value)?);

                self.inc_op();
            },

            
            Op::Store => {
                let Value::Usize(index) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };
                
                let value = self.pop()?;

                self.frame.insert(index, value);

                self.inc_op();
            },
            
            Op::GetFn => {
                let Value::Symbol(name) = self.pop()? else {
                    return Err(VmError::TypeCheck);
                };

                let Some(function) = self.functions.get(&name) else {
                    return Err(VmError::UnknownFunction(name));
                };

                self.stack.push(Value::Function((*function).clone()));
                self.inc_op();
            },


            Op::Call => {
                let Value::Function(function) = self.pop()? else {
                    unimplemented!()
                };

                let mut frame = BTreeMap::new();

                std::mem::swap(&mut frame, &mut self.frame);

                let call_value = CallStackEntry {
                    instruction: self.instruction_pointer + 1,
                    scope: frame,
                };

                self.call_stack.push(call_value);
                // self.frame set to BTreeMap::new() in swap
                self.instruction_pointer = function.offset;
            },

            Op::Return => {
                let Some(entry) = self.call_stack.pop() else {
                    return Err(VmError::InvalidOperation);
                };

                self.frame = entry.scope;
                self.instruction_pointer = entry.instruction;
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
                    (F32(a), F32(b)) => F32(a + b),
                    (F64(a), F64(b)) => F64(a + b),

                    (I32(a), I32(b)) => I32(a + b),
                    (I64(a), I64(b)) => I64(a + b),

                    (U32(a), U32(b)) => U32(a + b),
                    (U64(a), U64(b)) => U64(a + b),
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
           
            Value::Function(ptr) => Value::Function((*ptr).clone()),
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