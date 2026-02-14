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
#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    pub(crate) obj_type: ObjType,
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

#[derive(Debug)]
pub struct ObjUpvalue {
    pub obj: Box<Obj>,
    pub location: *mut Value,
    pub closed: Value,
    pub next: Option<Box<ObjUpvalue>>,
}
impl ObjUpvalue {
    pub fn new_open(location: *mut Value) -> Self {
        ObjUpvalue {
            obj: Box::new(Obj {
                obj_type: ObjType::ObjUpvalue,
                is_marked: false,
                next: None,
            }),
            location,
            closed: Value::Nil,
            next: None,
        }
    }
    pub fn new_closed(value: Value) -> Self {
        ObjUpvalue {
            obj: Box::new(Obj {
                obj_type: ObjType::ObjUpvalue,
                is_marked: false,
                next: None,
            }),
            location: std::ptr::null_mut(),
            closed: value,
            next: None,
        }
    }
    pub fn is_open(&self) -> bool {
        !self.location.is_null()
    }
    pub fn close(&mut self) {
        if self.is_open() {
            self.closed = unsafe { *self.location };
            self.location = std::ptr::null_mut();
        }
    }
}
impl ObjUpvalue {
    pub fn get_value(&self) -> &Value {
        if self.is_open() {
            unsafe { &*self.location }
        } else {
            &self.closed
        }
    }

    pub fn set_value(&mut self, value: Value) {
        if self.is_open() {
            unsafe {
                *self.location = value;
            }
        } else {
            self.closed = value;
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ObjClosure {
    obj: Box<Obj>,
    function: *const ObjFunction,
    upvalues: *const ObjUpvalue,
    upvalue_count: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ObjString {
    pub obj: Box<Obj>,
    pub data: String,
    pub hash: u64,
}
impl ObjString {
    pub fn from_string(s: String) -> *mut ObjString {
        let hash = hash_string(&s);

        let obj_string = Box::new(ObjString {
            obj: Box::new(Obj {
                obj_type: ObjType::ObjString,
                is_marked: false,
                next: None,
            }),
            data: s,
            hash,
        });

        Box::into_raw(obj_string)
    }

    pub fn copy_from_str(s: &str) -> *mut ObjString {
        Self::from_string(s.to_owned())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.data
    }

    /// Helper: length in bytes (not chars)
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

pub fn hash_string(s: &str) -> u64 {
    let mut hash: u64 = 2166136261u64;
    for &byte in s.as_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

pub unsafe fn as_obj_string(obj: *mut Obj) -> *mut ObjString {
    debug_assert!(!obj.is_null());
    debug_assert!((*obj).obj_type == ObjType::ObjString);
    obj.cast()
}

impl Obj {
    pub fn print(&self) {
        match self.obj_type {
            ObjType::ObjString => {
                let s = unsafe { as_obj_string(self as *const _ as *mut _) };
                let as_str = unsafe { (*s).as_str() };
                print!("\"{}\"", as_str);
            }
            ObjType::ObjFunction => { /* … */ }
            _ => print!("<unknown object>"),
        }
    }
}

impl std::fmt::Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            match self.obj_type {
                ObjType::ObjString => {
                    let s = as_obj_string(self as *const _ as *mut _);
                    let as_str = unsafe { (*s).as_str() };
                    write!(f, "\"{}\"", as_str)
                }
                _ => write!(f, "<obj {:?}>", self.obj_type),
            }
        }
    }
}
