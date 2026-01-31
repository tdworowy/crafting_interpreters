use crate::object::ObjFunction;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    String(String),
    Function(Box<ObjFunction>),
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl Value {
    pub fn as_function(&self) -> &ObjFunction {
        match self {
            Value::Function(f) => f,
            _ => panic!("Expected function, found {:?}", self),
        }
    }
}

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
    SetProperty(isize),
    GetProperty(isize),
    Call(isize),
    Invoke(isize, isize),
    SuperInvoke(isize, isize),
    Jump(i16),
    JumpIfFalse(i16),
    Loop(i16),
    Closure(isize),
    Method(isize),
    OpConstant,
    OpNil,
    OpTrue,
    OpFalse,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNot,
    OpEqual,
    OpGreater,
    OpLess,
    OpNegate,
    OpReturn,
    OpPrint,
    OpPop,
    Class(isize),
    OpInvoke,
    OpInherit,
    GetSuper(isize),
    OpCloseUpvalue,
    OpNop,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub count: i32,
    pub code: Vec<OpCode>,
    pub lines: Vec<usize>,
    pub constants: Vec<Value>,
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
    pub fn add_constant(&mut self, value: Value) -> isize {
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
    chunk.write_chunk(OpCode::OpAdd, 1);
    chunk.write_chunk(OpCode::OpPop, 2);
    chunk.write_chunk(OpCode::OpAdd, 3);

    let values: Vec<Value> = Vec::new();

    assert_eq!(chunk.count, 3);
    assert_eq!(
        chunk.code,
        vec![OpCode::OpAdd, OpCode::OpPop, OpCode::OpAdd]
    );
    assert_eq!(chunk.lines, vec![1, 2, 3]);
    assert_eq!(chunk.constants, values);
    chunk.add_constant(Value::String("123".to_owned()));
    assert_eq!(chunk.constants, vec![Value::String("123".to_owned())]);
}
