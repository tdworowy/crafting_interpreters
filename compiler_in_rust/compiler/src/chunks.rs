#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    Constant(isize),
    DefineGlobal(isize),
    Jump(i16),
    JumpIfFalse(i16),
    Loop(i16),
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
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub count: i32,
    pub code: Vec<OpCode>,
    pub lines: Vec<usize>,
    pub constants: Vec<String>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            count: 0,
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }
    pub fn write_chunk(&mut self, byte: OpCode, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
        self.count += 1;
    }
    pub fn add_constant(&mut self, value: String) -> isize {
        self.constants.push(value);
        self.constants.len() as isize - 1
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
    chunk.write_chunk(OpCode::OP_ADD, 1);
    chunk.write_chunk(OpCode::OP_CALL, 2);
    chunk.write_chunk(OpCode::OP_CLOSURE, 3);

    let strings: Vec<String> = Vec::new();

    assert_eq!(chunk.count, 3);
    assert_eq!(
        chunk.code,
        vec![OpCode::OP_ADD, OpCode::OP_CALL, OpCode::OP_CLOSURE]
    );
    assert_eq!(chunk.lines, vec![1, 2, 3]);
    assert_eq!(chunk.constants, strings);
    chunk.add_constant("123".to_owned());
    assert_eq!(chunk.constants, vec!["123"]);
}
