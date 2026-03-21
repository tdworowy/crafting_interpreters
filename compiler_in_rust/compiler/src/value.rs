use crate::object::{
    Obj, ObjBoundMethod, ObjClass, ObjClosure, ObjNative, ObjString, as_obj_bound_method,
    as_obj_class, as_obj_closure, as_obj_native, as_obj_string,
};
use std::fmt;

#[derive(Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(*mut Obj),
}

impl Value {
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_obj(&self) -> bool {
        matches!(self, Value::Obj(_))
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => panic!("Expected bool"),
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            _ => panic!("Expected number"),
        }
    }

    pub fn as_obj(&self) -> *mut Obj {
        match self {
            Value::Obj(ptr) => *ptr,
            _ => panic!("Expected object"),
        }
    }

    pub fn as_string(&self) -> *mut ObjString {
        match self {
            Value::Obj(ptr) => unsafe { as_obj_string(*ptr) },
            _ => panic!("Value is not a string"),
        }
    }

    pub fn as_closure(&self) -> *mut ObjClosure {
        match self {
            Value::Obj(ptr) => unsafe { as_obj_closure(*ptr) },
            _ => panic!("Value is not a closure"),
        }
    }

    pub fn as_native(&self) -> *mut ObjNative {
        match self {
            Value::Obj(ptr) => unsafe { as_obj_native(*ptr) },
            _ => panic!("Value is not a native function"),
        }
    }

    pub fn as_class(&self) -> *mut ObjClass {
        match self {
            Value::Obj(ptr) => unsafe { as_obj_class(*ptr) },
            _ => panic!("Value is not a class"),
        }
    }

    pub fn as_bound_method(&self) -> *mut ObjBoundMethod {
        match self {
            Value::Obj(ptr) => unsafe { as_obj_bound_method(*ptr) },
            _ => panic!("Value is not a bound method"),
        }
    }

    pub fn print(&self) {
        match self {
            Value::Bool(b) => print!("{}", if *b { "true" } else { "false" }),
            Value::Nil => print!("nil"),
            Value::Number(n) => print!("{n}"),
            Value::Obj(ptr) => unsafe {
                (**ptr).print();
            },
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "Bool({})", v),
            Value::Nil => write!(f, "Nil"),
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Obj(ptr) => write!(f, "Obj({:p})", ptr),
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
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) -> usize {
        let index = self.values.len();
        self.values.push(value);
        index
    }

    pub fn get(&self, index: usize) -> Value {
        self.values[index].clone()
    }
}

impl Default for ValueArray {
    fn default() -> Self {
        Self::new()
    }
}

// TODO make it work without pointers and unsafe
