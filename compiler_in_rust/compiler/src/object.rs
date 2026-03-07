use crate::chunks::Chunk;
use crate::value::Value;

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
pub type NativeFn = fn(arg_count: usize, args: &[Value]) -> Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    pub obj_type: ObjType,
    pub is_marked: bool,
}

impl Obj {
    pub fn new(obj_type: ObjType) -> Self {
        Self {
            obj_type,
            is_marked: false,
        }
    }

    pub fn print(&self) {
        match self.obj_type {
            ObjType::ObjString => print!("<string>"),
            ObjType::ObjFunction => print!("<fn>"),
            ObjType::ObjClosure => print!("<closure>"),
            ObjType::ObjNative => print!("<native fn>"),
            ObjType::ObjUpvalue => print!("<upvalue>"),
            ObjType::ObjClass => print!("<class>"),
            ObjType::ObjInstance => print!("<instance>"),
            ObjType::ObjBoundMethod => print!("<bound method>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjFunction {
    pub obj: Obj,
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk,
    pub name: String,
}

impl ObjFunction {
    pub fn new() -> Self {
        Self {
            obj: Obj::new(ObjType::ObjFunction),
            arity: 0,
            upvalue_count: 0,
            chunk: Chunk::new(),
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClosure {
    pub obj: Obj,
    pub function: ObjFunction,
    pub upvalues: Vec<ObjUpvalue>,
}

impl ObjClosure {
    pub fn new(function: ObjFunction) -> Self {
        Self {
            obj: Obj::new(ObjType::ObjClosure),
            function,
            upvalues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjUpvalue {
    pub obj: Obj,
    pub location: Option<Value>,
    pub closed: Value,
}

impl ObjUpvalue {
    pub fn new_open(value: Value) -> Self {
        Self {
            obj: Obj::new(ObjType::ObjUpvalue),
            location: Some(value),
            closed: Value::Nil,
        }
    }

    pub fn new_closed(value: Value) -> Self {
        Self {
            obj: Obj::new(ObjType::ObjUpvalue),
            location: None,
            closed: value,
        }
    }

    pub fn is_open(&self) -> bool {
        self.location.is_some()
    }

    pub fn close(&mut self) {
        if let Some(v) = self.location.take() {
            self.closed = v;
        }
    }

    pub fn get_value(&self) -> &Value {
        match &self.location {
            Some(v) => v,
            None => &self.closed,
        }
    }

    pub fn set_value(&mut self, value: Value) {
        match &mut self.location {
            Some(v) => *v = value,
            None => self.closed = value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjString {
    pub obj: Obj,
    pub data: String,
    pub hash: u64,
}

impl ObjString {
    pub fn from_string(s: String) -> Self {
        let hash = hash_string(&s);

        Self {
            obj: Obj::new(ObjType::ObjString),
            data: s,
            hash,
        }
    }

    pub fn copy_from_str(s: &str) -> Self {
        Self::from_string(s.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Clone)]
pub struct ObjNative {
    pub obj: Obj,
    pub function: NativeFn,
}

impl ObjNative {
    pub fn new(function: NativeFn) -> Self {
        Self {
            obj: Obj::new(ObjType::ObjNative),
            function,
        }
    }

    pub fn call(&self, arg_count: usize, args: &[Value]) -> Value {
        (self.function)(arg_count, args)
    }
}
pub fn hash_string(s: &str) -> u64 {
    let mut hash: u64 = 2166136261;

    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(16777619);
    }

    hash
}
pub unsafe fn as_obj_string(obj: *mut Obj) -> *mut ObjString {
    debug_assert!((*obj).obj_type == ObjType::ObjString);
    obj as *mut ObjString
}
