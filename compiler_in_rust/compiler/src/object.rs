use crate::chunks::Chunk;

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

pub struct Obj<'a> {
    obj_type: ObjType,
    is_marked: bool,
    next: &'a Obj<'a>,
}
pub struct ObjFunction<'a> {
    obj: Obj<'a>,
    arity: usize,
    upvalue_count: usize,
    pub chunk: Chunk,
    name: String,
}
