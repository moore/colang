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
            functions: FnTable::new(),
            types: BTreeMap::new(),
        };

        let mut vm = Vm::new(module);

        vm.run()?;
        dbg!(&vm.stack);
        assert!(vm.stack.len() == 1);

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
            functions: FnTable::new(),
            types: BTreeMap::new(),
        };

        let mut vm = Vm::new(module);

        vm.run()?;
        dbg!(&vm.stack);
        assert!(vm.stack.len() == 2);

        let value = match &vm.stack[1] {
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
            functions: FnTable::new(),
            types: BTreeMap::new(),
        };

        let mut vm = Vm::new(module);

        vm.run()?;
        dbg!(&vm.stack);
        assert!(vm.stack.len() == 5);

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
            Op::Halt,
            ];

        let module = Module {
            start: 0,
            code,
            functions: FnTable::new(),
            types: BTreeMap::new(),
        };

        let mut vm = Vm::new(module);

        vm.run()?;
        dbg!(&vm.stack);
        assert!(vm.stack.len() == 4);

        let value = match &vm.stack[1] {
            Value::U32(v) => Some(*v),
            _ => None,
        };

        assert!(value == Some(13));
        Ok(())
    }


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


    #[test]
    fn main_call () -> Result<(), VmError> {
        let code = vec![
            // fn 0
            Op::AddU32, 
            Op::Return,
            // start
            Op::None,   // Query Value
            Op::U32(0), // Query Key
            Op::Usize(2), // Struct Size
            Op::Struct, // Build query struct
            Op::Query,  // Run Query
            Op::Read,   // Read fn value
            Op::U32(5), // First Fn arg
            Op::U32(7), // Second Fn arg
            Op::Usize(1), // Ret count
            Op::Usize(2), // Arg count
            Op::Usize(6), // Function depth
            Op::CopyFrom,
            Op::Call,
            Op::Halt,
            ];

        let mut functions = FnTable::new();
        functions.add_fn(0,0);

        let module = Module {
            start: 2,
            code,
            functions,
            types: BTreeMap::new(),
        };

        let mut vm = Vm::new(module);

        vm.run()?;
        assert!(vm.stack.len() == 3);

        let value = match &vm.stack[2] {
            Value::U32(v) => Some(*v),
            _ => None,
        };

        assert!(value == Some(12));
        Ok(())
    }
