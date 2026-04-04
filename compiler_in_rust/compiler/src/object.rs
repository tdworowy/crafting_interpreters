use crate::chunks::Chunk;
use crate::value::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type NativeFn = fn(arg_count: usize, args: &[Value]) -> Value;

/* ================== OBJECT ================== */

#[derive(Debug, Clone, PartialEq)]
pub enum Obj {
    String(ObjString),
    Function(ObjFunction),
    Closure(ObjClosure),
    Native(ObjNative),
    Upvalue(ObjUpvalue),
    Class(ObjClass),
    Instance(ObjInstance),
    BoundMethod(ObjBoundMethod),
}

impl Obj {
    pub fn print(&self) {
        match self {
            Obj::String(s) => print!("{}", s.data),
            Obj::Function(_) => print!("<fn>"),
            Obj::Closure(_) => print!("<closure>"),
            Obj::Native(_) => print!("<native fn>"),
            Obj::Upvalue(_) => print!("<upvalue>"),
            Obj::Class(c) => print!("<class {}>", c.name),
            Obj::Instance(_) => print!("<instance>"),
            Obj::BoundMethod(_) => print!("<bound method>"),
        }
    }
}

/* ================== FUNCTION ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjFunction {
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk,
    pub name: String,
}

impl ObjFunction {
    pub fn new() -> Self {
        Self {
            arity: 0,
            upvalue_count: 0,
            chunk: Chunk::new(),
            name: String::new(),
        }
    }
}

/* ================== CLOSURE ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClosure {
    pub function: Rc<ObjFunction>,
    pub upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
}

impl ObjClosure {
    pub fn new(function: Rc<ObjFunction>) -> Self {
        Self {
            function,
            upvalues: Vec::new(),
        }
    }
}

/* ================== UPVALUE ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjUpvalue {
    pub location: Option<Value>,
    pub closed: Value,
}

impl ObjUpvalue {
    pub fn new_open(value: Value) -> Self {
        Self {
            location: Some(value),
            closed: Value::Nil,
        }
    }

    pub fn new_closed(value: Value) -> Self {
        Self {
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
        self.location.as_ref().unwrap_or(&self.closed)
    }

    pub fn set_value(&mut self, value: Value) {
        match &mut self.location {
            Some(v) => *v = value,
            None => self.closed = value,
        }
    }
}

/* ================== STRING ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjString {
    pub data: String,
    pub hash: u64,
}

impl ObjString {
    pub fn from_string(s: String) -> Self {
        let hash = hash_string(&s);
        Self { data: s, hash }
    }

    pub fn copy_from_str(s: &str) -> Self {
        Self::from_string(s.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }
}

/* ================== NATIVE ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjNative {
    pub function: NativeFn,
}

impl ObjNative {
    pub fn new(function: NativeFn) -> Self {
        Self { function }
    }

    pub fn call(&self, arg_count: usize, args: &[Value]) -> Value {
        (self.function)(arg_count, args)
    }
}

/* ================== CLASS ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjClass {
    pub name: String,
    pub methods: HashMap<String, Value>,
}

impl ObjClass {
    pub fn new(name: String, methods: HashMap<String, Value>) -> Self {
        Self { name, methods }
    }
}

/* ================== INSTANCE ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjInstance {
    pub klass: Rc<ObjClass>,
    pub fields: HashMap<String, Value>,
}

impl ObjInstance {
    pub fn new(klass: Rc<ObjClass>) -> Self {
        Self {
            klass,
            fields: HashMap::new(),
        }
    }
}

/* ================== BOUND METHOD ================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub receiver: Value,
    pub method: Rc<ObjClosure>,
}

/* ================== HASH ================== */

pub fn hash_string(s: &str) -> u64 {
    let mut hash: u64 = 2166136261;

    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(16777619);
    }

    hash
}
