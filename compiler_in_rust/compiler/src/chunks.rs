use crate::{grow_capacity, value::ValueArray, vm::VM};

pub enum OpCode {
    OP_CONSTANT,
    OP_NIL,
    OP_TRUE,
    OP_FALSE,
    OP_ADD,
    OP_SUBTRACT,
    OP_MULTIPLY,
    OP_DIVIDE,
    OP_NOT,
    OP_EQUAL,
    OP_GREATER,
    OP_LESS,
    OP_NEGATE,
    OP_RETURN,
    OP_PRINT,
    OP_JUMP,
    OP_JUMP_IF_FALSE,
    OP_POP,
    OP_DEFINE_GLOBAL,
    OP_GET_LOCAL,
    OP_SET_LOCAL,
    OP_GET_GLOBAL,
    OP_SET_GLOBAL,
    OP_GET_UPVALUE,
    OP_SET_UPVALUE,
    OP_CLOSE_UPVALUE,
    OP_LOOP,
    OP_CALL,
    OP_CLOSURE,
    OP_CLASS,
    OP_SET_PROPERTY,
    OP_GET_PROPERTY,
    OP_METHOD,
    OP_INVOKE,
    OP_INHERIT,
    OP_GET_SUPER,
    OP_SUPER_INVOKE,
}
#[derive(Debug)]
pub struct Chunk {
    pub count: i32,
    pub capacity: i32,
    pub code: Vec<u8>,
    pub lines: Vec<i32>,
    pub constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            count: 0,
            capacity: 0,
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }
    pub fn write_chunk(&mut self, byte: u8, line: i32) {
        if self.capacity < self.count + 1 {
            self.capacity = grow_capacity!(self.capacity);
            self.code.resize(self.capacity as usize, 0);
            self.lines.resize(self.capacity as usize, 0);
        }
        self.code[self.count as usize] = byte;
        self.lines[self.count as usize] = line;
        self.count += 1;
    }
    pub fn add_constant(&mut self, value: u64, vm: &mut VM) -> usize {
        vm.push(value);
        self.constants.write(value);
        vm.pop();
        self.constants.values.len() - 1
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_chunk() {
    let mut chunk = Chunk::new();
    let mut vm = VM::new();
    chunk.write_chunk(10, 122);

    assert_eq!(chunk.count, 1);
    assert_eq!(chunk.capacity, 8);
    assert_eq!(chunk.code, vec![10, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(chunk.lines, vec![122, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        chunk.constants,
        ValueArray {
            count: 0,
            capacity: 0,
            values: vec![]
        }
    );
    chunk.add_constant(123, &mut vm);
    assert_eq!(
        chunk.constants,
        ValueArray {
            count: 1,
            capacity: 8,
            values: vec![123, 0, 0, 0, 0, 0, 0, 0]
        }
    );
}
