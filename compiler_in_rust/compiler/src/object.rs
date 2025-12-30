use crate::chunks::Chunk;
#[derive(Clone)]
pub enum ObjType {
    OBJ_STRING,
    OBJ_CLOSURE,
    OBJ_FUNCTION,
    OBJ_NATIVE,
    OBJ_UPVALUE,
    OBJ_CLASS,
    OBJ_INSTANCE,
    OBJ_BOUND_METHOD,
}
#[derive(Clone)]
pub struct Obj {
    obj_type: ObjType,
    is_marked: bool,
    next: Option<Box<Obj>>,
}
#[derive(Clone)]
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
                obj_type: ObjType::OBJ_FUNCTION,
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
