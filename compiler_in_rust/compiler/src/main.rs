use crate::vm::VM;

mod chunks;
mod compiler;
mod memory;
mod object;
mod scanner;
mod value;
mod vm;

fn main() {
    let mut vm = VM::new();
}
