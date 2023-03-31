use super::*;


#[derive(Debug)]
pub enum TestError {
    LangError(LangError),
    VmError(VmError),
}

impl From<VmError> for TestError {
    fn from(value: VmError) -> Self {
        TestError::VmError(value)
    }
}

impl From<LangError> for TestError {
    fn from(value: LangError) -> Self {
        TestError::LangError(value)
    }
}

#[test]
fn call_fn () -> Result<(), TestError> {
    let file = "src/lang/example.co";
    let module = parse_colang_file(file)?;
    let mut vm = Vm::new(module);
    vm.run()?;

    Ok(())
}


#[test]
fn simple () -> Result<(), TestError> {
    let file = "src/lang/simple.co";
 
    let module = parse_colang_file(file)?;
    dbg!(&module);
    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(vm.stack());
    assert!(vm.stack_len() == 1);

    let computed = &vm.stack_get(0);
    let value = match computed {
        Some(Value::I64(v)) => Some(*v),
        _ => None,
    };
    assert!(value == Some(12));
    
    Ok(())
}


#[test]
fn simple_vars () -> Result<(), TestError> {
    let file = "src/lang/simple_vars.co";
 
    let module = parse_colang_file(file)?;
    dbg!(&module);
    let mut vm = Vm::new(module);

    vm.run()?;
    dbg!(vm.code());
    dbg!(vm.stack());
    assert!(vm.stack_len() == 1);

    let computed = &vm.stack_get(0);
    let value = match computed {
        Some(Value::I64(v)) => Some(*v),
        _ => None,
    };
    assert!(value == Some(12));
    
    Ok(())
}
