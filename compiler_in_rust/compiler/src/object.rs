use crate::chunks::Chunk;
#[derive(Debug, Clone, PartialEq)]
pub enum ObjType {
    ObjString,
    ObjClosure,
    ObjFunction,
    ObjNative,
    ObjUpvalue,
    ObjClass,
    ObjInstance,
    ObjBoundMethod,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    obj_type: ObjType,
    is_marked: bool,
    next: Option<Box<Obj>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ObjFunction {
    obj: Box<Obj>,
    pub arity: usize,
    pub upvalue_count: isize,
    pub chunk: Chunk,
    pub name: String,
}

impl ObjFunction {
    pub fn new() -> Self {
        ObjFunction {
            obj: Box::new(Obj {
                obj_type: ObjType::ObjFunction,
                is_marked: false,
                next: None,
            }),
            arity: 0,
            upvalue_count: 0,
            chunk: Chunk::new(),
            name: "".to_owned(),
        }
    }
}
