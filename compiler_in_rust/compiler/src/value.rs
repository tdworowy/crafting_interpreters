use std::fmt;

use crate::object::{Obj, ObjClosure, ObjFunction, ObjString, ObjType};

#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(*mut Obj),
}

impl Value {
    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    #[inline]
    pub fn is_obj(&self) -> bool {
        matches!(self, Value::Obj(_))
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        self.is_obj() && unsafe { (*self.as_obj()).obj_type == ObjType::ObjString }
    }

    #[inline]
    pub fn is_function(&self) -> bool {
        self.is_obj() && unsafe { (*self.as_obj()).obj_type == ObjType::ObjFunction }
    }

    #[inline]
    pub fn is_closure(&self) -> bool {
        self.is_obj() && unsafe { (*self.as_obj()).obj_type == ObjType::ObjClosure }
    }

    #[inline]
    pub fn is_upvalue(&self) -> bool {
        self.is_obj() && unsafe { (*self.as_obj()).obj_type == ObjType::ObjUpvalue }
    }

    #[inline]
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => panic!("Expected bool"),
        }
    }

    #[inline]
    pub fn as_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            _ => panic!("Expected number"),
        }
    }

    #[inline]
    pub fn as_obj(&self) -> *mut Obj {
        match self {
            Value::Obj(ptr) => *ptr,
            _ => panic!("Expected object"),
        }
    }

    #[inline]
    pub fn as_string(&self) -> *mut ObjString {
        self.as_obj_typed(ObjType::ObjString)
    }

    #[inline]
    pub fn as_function(&self) -> *mut ObjFunction {
        self.as_obj_typed(ObjType::ObjFunction)
    }

    #[inline]
    pub fn as_closure(&self) -> *mut ObjClosure {
        self.as_obj_typed(ObjType::ObjClosure)
    }

    #[inline]
    fn as_obj_typed<T>(&self, expected_type: ObjType) -> *mut T {
        let obj = self.as_obj();
        unsafe {
            if (*obj).obj_type != expected_type {
                panic!("Wrong object type");
            }
            obj.cast()
        }
    }

    pub fn print(&self) {
        match self {
            Value::Bool(b) => print!("{}", if *b { "true" } else { "false" }),
            Value::Nil => print!("nil"),
            Value::Number(n) => print!("{n}"),
            Self::Obj(ptr) => unsafe { ptr.as_ref().unwrap_unchecked().print() },
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Obj(ptr) => unsafe { write!(f, "{:?}", **ptr) },
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "Bool({b})"),
            Value::Nil => write!(f, "Nil"),
            Value::Number(n) => write!(f, "Number({n})"),
            Value::Obj(ptr) => unsafe {
                if ptr.is_null() {
                    write!(f, "Obj(NULL)")
                } else {
                    write!(f, "Obj({:?} → {:?})", *ptr, **ptr)
                }
            },
        }
    }
}

pub fn bool_val(value: bool) -> Value {
    Value::Bool(value)
}
pub fn nil_val() -> Value {
    Value::Nil
}
pub fn number_val(value: f64) -> Value {
    Value::Number(value)
}
pub fn obj_val(obj: *mut Obj) -> Value {
    Value::Obj(obj)
}

#[derive(Debug, Clone)]
pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        ValueArray { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) -> usize {
        let idx = self.values.len();
        self.values.push(value);
        idx
    }

    pub fn get(&self, index: usize) -> Value {
        self.values[index]
    }
}

impl Default for ValueArray {
    fn default() -> Self {
        Self::new()
    }
}
