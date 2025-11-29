use crate::{chunks::Chunk, vm::VM};

mod chunks;
mod compiler;
mod memory;
mod object;
mod scaner;
mod value;
mod vm;
fn main() {
    // TESTING
    let mut chunk = Chunk::new();
    let mut vm = VM::new();
    chunk.write_chunk(10, 122);
    chunk.add_constant(123, &mut vm);
    println!("{:?}", chunk);
}
