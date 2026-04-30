use crate::value::Value;

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
    pub count: isize,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Obj, ObjString};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn chunk_new_is_empty() {
        let chunk = Chunk::new();
        assert_eq!(chunk.count, 0);
        assert!(chunk.code.is_empty());
        assert!(chunk.lines.is_empty());
        assert!(chunk.constants.is_empty());
    }

    #[test]
    fn chunk_default_is_new() {
        let chunk = Chunk::default();
        assert_eq!(chunk, Chunk::new());
    }

    #[test]
    fn write_chunk_appends_code_and_line_and_increments_count() {
        let mut chunk = Chunk::new();

        chunk.write_chunk(OpCode::OpAdd, 10);
        chunk.write_chunk(OpCode::OpPop, 11);
        chunk.write_chunk(OpCode::Constant(123), 12);

        assert_eq!(chunk.count, 3);
        assert_eq!(
            chunk.code,
            vec![OpCode::OpAdd, OpCode::OpPop, OpCode::Constant(123)]
        );
        assert_eq!(chunk.lines, vec![10, 11, 12]);

        // Ensure code/lines stay in sync.
        assert_eq!(chunk.code.len(), chunk.lines.len());
        assert_eq!(chunk.code.len() as isize, chunk.count);
    }

    #[test]
    fn add_constant_returns_index_and_stores_value() {
        let mut chunk = Chunk::new();
        let str_obj1 = ObjString::from_string("hello".to_owned());
        let str_value1 = Value::Obj(Rc::new(RefCell::new(Obj::String(str_obj1))));
        let idx0 = chunk.add_constant(Value::Number(1.25));
        let idx1 = chunk.add_constant(str_value1);
        let idx2 = chunk.add_constant(Value::Nil);

        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);

        let str_obj2 = ObjString::from_string("hello".to_owned());
        let str_value2 = Value::Obj(Rc::new(RefCell::new(Obj::String(str_obj2))));

        assert_eq!(
            chunk.constants,
            vec![Value::Number(1.25), str_value2, Value::Nil]
        );
    }
}
