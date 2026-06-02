use crate::object::{Obj, ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjNative, ObjString};
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

    pub fn as_string(&self) -> Rc<ObjString> {
        let obj = self.as_obj();
        match &*obj.borrow() {
            Obj::String(string) => Rc::new(string.clone()),
            _ => panic!("Value is not a closure"),
        }
    }

    pub fn as_closure(&self) -> Rc<ObjClosure> {
        let obj = self.as_obj();

        match &*obj.borrow() {
            Obj::Closure(closure) => Rc::new(closure.clone()),
            _ => panic!("Value is not a closure"),
        }
    }

    pub fn as_native(&self) -> Rc<ObjNative> {
        let obj = self.as_obj();
        match &*obj.borrow() {
            Obj::Native(native) => Rc::new(native.clone()),
            _ => panic!("Value is not a native"),
        }
    }

    pub fn as_class(&self) -> Rc<ObjClass> {
        let obj = self.as_obj();
        match &*obj.borrow() {
            Obj::Class(klass) => Rc::clone(klass),
            _ => panic!("Value is not a Class"),
        }
    }

    pub fn as_bound_method(&self) -> Rc<ObjBoundMethod> {
        let obj = self.as_obj();
        match &*obj.borrow() {
            Obj::BoundMethod(bound_method) => Rc::new(bound_method.clone()),
            _ => panic!("Value is not a boundMethod"),
        }
    }
    pub fn as_function(&self) -> Rc<ObjFunction> {
        let obj = self.as_obj();
        match &*obj.borrow() {
            Obj::Function(function) => Rc::new(function.clone()),
            _ => panic!("Value is not a function"),
        }
    }

    pub fn print(&self) {
        match self {
            Value::Bool(b) => println!("{}", if *b { "true" } else { "false" }),
            Value::Nil => println!("nil"),
            Value::Number(n) => println!("{n}"),
            Value::Obj(obj) => {
                let obj = obj.borrow();
                match &*obj {
                    Obj::String(s) => println!("<string>{}<string>", &s.data),
                    Obj::Closure(_) => println!("<closure>"),
                    Obj::Function(fun) => println!("<function>{:?}<function>", fun.name),
                    Obj::Native(_) => println!("<native fn>"),
                    Obj::Class(klass) => println!("<class>{:?}<class>", klass.name),
                    Obj::BoundMethod(_) => println!("<bound method>"),
                    _ => {
                        println!("<unknown>");
                    }
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
