use crate::object::Obj;
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Rc<RefCell<Obj>>),
}

/* ==== Value impl ==== */
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

    pub fn as_obj(&self) -> Rc<RefCell<Obj>> {
        match self {
            Value::Obj(obj) => Rc::clone(obj),
            _ => panic!("Expected object"),
        }
    }

    pub fn as_string(&self) -> Rc<RefCell<Obj>> {
        let obj = self.as_obj();
        if matches!(*obj.borrow(), Obj::String(_)) {
            obj
        } else {
            panic!("Value is not a string");
        }
    }

    pub fn as_closure(&self) -> Rc<RefCell<Obj>> {
        let obj = self.as_obj();
        if matches!(*obj.borrow(), Obj::Closure(_)) {
            obj
        } else {
            panic!("Value is not a closure");
        }
    }

    pub fn as_native(&self) -> Rc<RefCell<Obj>> {
        let obj = self.as_obj();
        if matches!(*obj.borrow(), Obj::Native(_)) {
            obj
        } else {
            panic!("Value is not a native");
        }
    }

    pub fn as_class(&self) -> Rc<RefCell<Obj>> {
        let obj = self.as_obj();
        if matches!(*obj.borrow(), Obj::Class(_)) {
            obj
        } else {
            panic!("Value is not a class");
        }
    }

    pub fn as_bound_method(&self) -> Rc<RefCell<Obj>> {
        let obj = self.as_obj();
        if matches!(*obj.borrow(), Obj::BoundMethod(_)) {
            obj
        } else {
            panic!("Value is not a bound method");
        }
    }

    pub fn print(&self) {
        match self {
            Value::Bool(b) => print!("{}", if *b { "true" } else { "false" }),
            Value::Nil => print!("nil"),
            Value::Number(n) => print!("{n}"),
            Value::Obj(obj) => {
                let obj = obj.borrow();
                match &*obj {
                    Obj::String(s) => print!("{}", &s.data),
                    Obj::Closure(_) => print!("<closure>"),
                    Obj::Native(_) => print!("<native fn>"),
                    Obj::Class(_) => print!("<class>"),
                    Obj::BoundMethod(_) => print!("<bound method>"),
                    _ => {}
                }
            }
        }
    }
}

/* ==== Debug ==== */
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "Bool({})", v),
            Value::Nil => write!(f, "Nil"),
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Obj(_) => write!(f, "Obj(...)"),
        }
    }
}

/* ==== Constructors ==== */
pub fn bool_val(value: bool) -> Value {
    Value::Bool(value)
}

pub fn nil_val() -> Value {
    Value::Nil
}

pub fn number_val(value: f64) -> Value {
    Value::Number(value)
}

pub fn obj_val(obj: Obj) -> Value {
    Value::Obj(Rc::new(RefCell::new(obj)))
}

/* ==== ValueArray ==== */
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
