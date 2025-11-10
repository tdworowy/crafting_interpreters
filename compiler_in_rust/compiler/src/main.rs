use crate::chunks::Chunk;

mod chunks;
mod memory;
mod value;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write_chunk(10, 122);
    chunk.add_constant(123);
    println!("{:?}", chunk);
}
