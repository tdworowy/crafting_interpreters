use crate::object::ObjNative;
use crate::vm::clock_native;
use crate::{chunks::Chunk, vm::VM};

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
