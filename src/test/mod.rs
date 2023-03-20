    use super::*;

    #[test]
    fn struct_pop () -> Result<(), VmError> {
        let code = vec![
            Op::U32(1), 
            Op::U32(5), 
            Op::U32(7),
            Op::U32(3), 
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
            Op::U32(3), 
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
            Op::U32(2), 
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
            Op::U32(3), 
            Op::Struct,
            Op::U32(13), 
            Op::U32(17),
            Op::U32(2), 
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
            Op::U32(2), // Struct Size
            Op::Struct, // Build query struct
            Op::Query,  // Run Query
            Op::Read,   // Read fn value
            Op::U32(5),
            Op::Swap,
            Op::U32(7),
            Op::Swap,
            Op::U32(1),
            Op::Swap,
            Op::U32(2),
            Op::Swap,
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
        assert!(vm.stack.len() == 2);

        let value = match &vm.stack[1] {
            Value::U32(v) => Some(*v),
            _ => None,
        };

        assert!(value == Some(12));
        Ok(())
    }
