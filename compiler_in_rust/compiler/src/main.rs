use crate::{chunks::Chunk, vm::VM};

mod chunks;
mod compiler;
mod memory;
mod object;
mod scaner;
mod vm;
fn main() {
    // TESTING
    let mut chunk = Chunk::new();
    let mut vm = VM::new();
    chunk.write_chunk(chunks::OpCode::OP_ADD, 122);
    println!("{:?}", chunk);
}
