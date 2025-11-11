pub struct VM {
    pub stack_top: u64,
    pub stack: Vec<u64>,
}
impl VM {
    pub fn new() -> Self {
        Self {
            stack_top: 0,
            stack: vec![],
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
