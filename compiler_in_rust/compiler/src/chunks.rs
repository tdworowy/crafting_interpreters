#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    Constant(isize),
    DefineGlobal(isize),
    DefineLocal(isize),
    SetGlobal(isize),
    SetLocal(isize),
    GetGlobal(isize),
    GetLocal(isize),
    GetUpvalue(isize),
    SetUpvalue(isize),
    Call(isize),
    Invoke(isize, isize),
    SuperInvoke(isize, isize),
    Jump(i16),
    JumpIfFalse(i16),
    Loop(i16),
    Closure(isize),
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
    OP_POP,
    OP_DEFINE_GLOBAL,
    OP_CLASS,
    OP_SET_PROPERTY,
    OP_GET_PROPERTY,
    OP_METHOD,
    OP_INVOKE,
    OP_INHERIT,
    OP_GET_SUPER,
    OP_CLOSE_UPVALUE,
    OP_NOP,
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

// TODO better tests
#[test]
fn test_chunk() {
    let mut chunk = Chunk::new();
    chunk.write_chunk(OpCode::OP_ADD, 1);
    chunk.write_chunk(OpCode::OP_POP, 2);
    chunk.write_chunk(OpCode::OP_ADD, 3);

    let strings: Vec<String> = Vec::new();

    assert_eq!(chunk.count, 3);
    assert_eq!(
        chunk.code,
        vec![OpCode::OP_ADD, OpCode::OP_POP, OpCode::OP_ADD]
    );
    assert_eq!(chunk.lines, vec![1, 2, 3]);
    assert_eq!(chunk.constants, strings);
    chunk.add_constant("123".to_owned());
    assert_eq!(chunk.constants, vec!["123"]);
}
