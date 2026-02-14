use crate::object::{Obj, ObjClosure, ObjString, ObjUpvalue};
use crate::value::Value;
use std::collections::HashMap;

pub struct VM {
    call_frames: Vec<CallFrame>,
    stack: Vec<u64>,
    stack_top: u64,
    globals: HashMap<String, Value>,
    strings: HashMap<String, Value>,
    init_string: ObjString,
    open_upvalue: Option<ObjUpvalue>,
    bytes_allocated: usize,
    next_gc: usize,
    objects: Vec<Obj>,
    gray_count: usize,
    gray_capacity: usize,
    gray_stack: Vec<Obj>,
}
struct CallFrame {
    ip: usize,
    closure: ObjClosure,
    slots: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack_top: 0,
            globals: Default::default(),
            strings: Default::default(),
            init_string: ObjString::from_string("init".to_owned()),
            open_upvalue: None,
            bytes_allocated: 0,
            next_gc: 0,
            objects: vec![],
            gray_count: 0,
            gray_capacity: 0,
            stack: vec![],
            call_frames: vec![],
            gray_stack: vec![],
        }
    }
    pub fn push(&mut self, value: u64) {
        self.stack.push(value);
        self.stack_top += 1;
    }
    pub fn pop(&mut self) -> u64 {
        self.stack_top -= 1;
        self.stack.pop().expect("Can't pop value from stack")
    }
    fn peek(&mut self, distance: i64) -> u64 {
        self.stack[self.stack_top as usize - distance as usize]
    }
}

#[test]
fn test_vm() {
    let mut vm = VM::new();
    assert_eq!(vm.stack_top, 0);
    assert_eq!(vm.stack, vec![]);

    vm.push(123);
    vm.push(124);

    assert_eq!(vm.stack_top, 2);
    assert_eq!(vm.stack, vec![123, 124]);

    vm.pop();

    assert_eq!(vm.stack_top, 1);
    assert_eq!(vm.stack, vec![123]);
}
