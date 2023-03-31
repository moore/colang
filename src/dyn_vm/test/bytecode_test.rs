use super::*;

#[test]
fn struct_pop () -> Result<(), VmError> {
    let code = vec![
        Op::U32(1), 
        Op::U32(5), 
        Op::U32(7),
        Op::Usize(3), 
        Op::Struct,
        Op::Pop,
        Op::Halt,
        ];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(&vm.stack);
    assert!(vm.stack.len() == 0);

    Ok(())
}

#[test]
fn struct_copy () -> Result<(), VmError> {
    let code = vec![
        Op::U32(1), 
        Op::U32(5), 
        Op::U32(7),
        Op::Usize(3), 
        Op::Struct,
        Op::Copy,
        Op::Halt,
        ];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(&vm.stack);
    assert!(vm.stack.len() == 2);
    Ok(())
}

#[test]
fn value_copy () -> Result<(), VmError> {
    let code = vec![
        Op::U32(11), 
        Op::Copy,
        Op::Halt,
        ];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(&vm.stack);
    assert!(vm.stack.len() == 2);
    assert!(Vm::eq_value(&vm.stack[0], &vm.stack[1])?);

    Ok(())
}

#[test]
fn struct_swap1 () -> Result<(), VmError> {
    let code = vec![
        Op::U32(1), 
        Op::U32(5), 
        Op::U32(7),
        Op::Usize(3), 
        Op::Struct,
        Op::U32(11),
        Op::Swap,
        Op::Pop,
        Op::Halt,
        ];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(&vm.stack);
    assert!(vm.stack.len() == 1);

    let value = match &vm.stack[0] {
        Value::U32(v) => Some(*v),
        _ => None,
    };

    assert!(value == Some(11));
    Ok(())
}


#[test]
fn struct_swap2 () -> Result<(), VmError> {
    let code = vec![
        Op::U32(11),

        Op::U32(13), 
        Op::U32(17),
        Op::Usize(2), 
        Op::Struct,
        Op::Swap,
        Op::Halt,
        ];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(&vm.stack);
    assert!(vm.stack.len() == 2);

    let value = match vm.stack.last() {
        Some(Value::U32(v)) => Some(*v),
        _ => None,
    };

    assert!(value == Some(11));
    Ok(())
}


#[test]
fn struct_swap3 () -> Result<(), VmError> {
    let code = vec![
        Op::U32(1), 
        Op::U32(5), 
        Op::U32(7),
        Op::Usize(3), 
        Op::Struct,
        Op::U32(13), 
        Op::U32(17),
        Op::Usize(2), 
        Op::Struct,
        Op::Swap,
        Op::Pop,
        Op::Usize(1),
        Op::StructRead,
        Op::Halt,
        ];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(&vm.stack);
    assert!(vm.stack.len() == 2);

    let value = match &vm.stack[1] {
        Value::U32(v) => Some(*v),
        _ => None,
    };

    assert!(value == Some(13));
    Ok(())
}


#[test]
fn main_test () -> Result<(), VmError> {
    let code = vec![Op::U32(5), Op::U32(7), Op::Add, Op::Halt];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    assert!(vm.stack.len() == 1);

    let value = match &vm.stack[0] {
        Value::U32(v) => Some(*v),
        _ => None,
    };

    assert!(value == Some(12));
    Ok(())
}


#[test]
fn main_call () -> Result<(), VmError> {
    let code = vec![
        // fn 0
        Op::Add, 
        Op::Return,
        // start
        Op::U32(5), // First Fn arg
        Op::U32(7), // Second Fn arg
        Op::Symbol("add".to_string()),
        Op::GetFn,
        Op::Call,
        Op::Halt,
        ];

    let mut functions = BTreeMap::new();
    let info = FnInfo {
        ptr: 0,
        args: Vec::new(),
    };

    functions.insert("add".to_string(),info);

    let module = Module {
        start: 2,
        code,
        functions,
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    assert!(vm.stack.len() == 1);

    let value = match &vm.stack[0] {
        Value::U32(v) => Some(*v),
        _ => None,
    };

    assert!(value == Some(12));
    Ok(())
}
