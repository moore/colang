use super::*;

/* Needs to read the type table for struct to consume type
#[test]
fn struct_test () {
    let code = vec![Op::U32(1), Op::U32(5), Op::U32(7), Op::Struct];

    let module = Module {
        start: 0,
        code,
        functions: BTreeMap<Fn,Ins>,
        types: BTreeMap<Typ,Vec<Value>>,
    }
}
*/



#[test]
fn main_test () -> Result<(), VmError> {
    let code = vec![Op::U32(5), Op::U32(7), Op::AddU32, Op::Halt];

    let module = Module {
        start: 0,
        code,
        functions: FnTable::new(),
        types: BTreeMap::new(),
    };

    let mut vm = Vm::new(module);

    vm.run()?;
    assert!(vm.stack.len() == 2);

    let value = match &vm.stack[1] {
        Value::U32(v) => Some(*v),
        _ => None,
    };

    assert!(value == Some(12));
    Ok(())
}
