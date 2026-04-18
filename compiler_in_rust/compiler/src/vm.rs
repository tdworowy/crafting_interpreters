use crate::object::{
    NativeFn, Obj, ObjBoundMethod, ObjClass, ObjClosure, ObjInstance, ObjNative, ObjString,
    ObjUpvalue,
};
use crate::value::{Value, obj_val};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct VM {
    call_frames: Vec<CallFrame>,
    stack: Vec<Value>,
    stack_top: usize,
    globals: HashMap<String, Value>,
    strings: HashMap<String, Value>,
    init_string: ObjString,
    open_upvalues: Vec<Rc<ObjUpvalue>>,
    bytes_allocated: usize,
    next_gc: usize,
    objects: Vec<Obj>,
    gray_count: usize,
    gray_capacity: usize,
    gray_stack: Vec<Obj>,
}
pub struct CallFrame {
    pub closure: Rc<ObjClosure>,
    pub ip: usize,
    pub slot_start: usize,
}

pub fn clock_native(_: usize, _: &[Value]) -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();

    Value::Number(now)
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            stack_top: 0,
            globals: Default::default(),
            strings: Default::default(),
            init_string: ObjString::from_string("init".to_owned()),
            open_upvalues: vec![],
            bytes_allocated: 0,
            next_gc: 0,
            objects: vec![],
            gray_count: 0,
            gray_capacity: 0,
            stack: vec![],
            call_frames: vec![],
            gray_stack: vec![],
        };
        vm.define_native("clock", clock_native);
        vm
    }
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
        self.stack_top += 1;
    }
    pub fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack.pop().expect("Can't pop value from stack")
    }
    fn peek(&mut self, distance: i64) -> Value {
        self.stack[self.stack_top - distance as usize].to_owned()
    }
    fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.stack = vec![];
        self.open_upvalues = vec![];
    }
    fn runtime_error(&mut self, msg: String) {
        eprintln!("{msg}");

        for frame in self.call_frames.iter().rev() {
            let frame = frame.to_owned();
            let function = &frame.closure.function;

            let instruction_idx = frame.ip.saturating_sub(1);
            let line = function.chunk.lines[instruction_idx];
            eprint!("[line {line}] in ");

            if function.name.is_empty() {
                eprintln!("<script>");
            } else {
                eprintln!("{}", function.name);
            }
        }

        self.reset_stack();
    }
    fn runtime_error_fmt(&mut self, fmt: &str, args: std::fmt::Arguments<'_>) {
        self.runtime_error(format!("{fmt}{args}"));
    }
    pub fn define_native(&mut self, name: &str, function: NativeFn) {
        let name_obj = obj_val(Obj::String(ObjString::copy_from_str(name)));
        let native_obj = obj_val(Obj::Native(ObjNative::new(function)));

        self.push(name_obj.clone());
        self.push(native_obj.clone());

        let key = match &*name_obj.as_obj().borrow() {
            Obj::String(s) => s.data.clone(),
            _ => panic!("Expected string as key"),
        };

        self.globals.insert(key, native_obj);

        self.pop();
        self.pop();
    }
    fn call(&mut self, closure: Rc<ObjClosure>, arg_count: usize) -> bool {
        if arg_count != closure.function.arity {
            self.runtime_error(format!(
                "Expected {} arguments but got {}.",
                closure.function.arity, arg_count
            ));
            return false;
        }

        if self.call_frames.len() >= 64 {
            self.runtime_error("Stack overflow.".to_string());
            return false;
        }

        let frame = CallFrame {
            closure: closure.clone(),
            ip: 0, // ✅ start at beginning
            slot_start: self.stack.len() - arg_count - 1,
        };

        self.call_frames.push(frame);
        true
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        let obj_rc = match callee {
            Value::Obj(ref o) => o.clone(),
            _ => {
                self.runtime_error("Can only call functions and classes.".to_owned());
                return false;
            }
        };

        let obj = obj_rc.borrow();

        match &*obj {
            Obj::Class(klass) => {
                // Create instance
                let instance = Obj::Instance(ObjInstance::new(Rc::new(klass.clone())));
                let instance_val = obj_val(instance);

                // Replace callee slot with instance
                let slot = self.stack_top - arg_count - 1;
                self.stack[slot] = instance_val.clone();

                // Call initializer if exists
                if let Some(init) = klass.methods.get("init") {
                    self.call_value(init.clone(), arg_count)
                } else {
                    if arg_count != 0 {
                        self.runtime_error(format!("Expected 0 arguments, got {}.", arg_count));
                        return false;
                    }
                    true
                }
            }

            Obj::Closure(c) => self.call(Rc::from(c.clone()), arg_count),

            Obj::Native(native) => {
                let args_start = self.stack_top - arg_count;
                let result = (native.function)(arg_count, &self.stack[args_start..self.stack_top]);
                // Pop args + callee, push result
                self.stack_top -= arg_count + 1;
                self.push(result);

                true
            }

            Obj::BoundMethod(bound) => {
                // Replace callee with receiver
                let slot = self.stack_top - arg_count - 1;
                self.stack[slot] = bound.receiver.clone();

                let method = Value::Obj(Rc::new(std::cell::RefCell::new(Obj::Closure(
                    (*bound.method).clone(),
                ))));

                drop(obj);
                self.call_value(method, arg_count)
            }

            _ => {
                self.runtime_error("Can only call functions and classes.".to_owned());
                false
            }
        }
    }
    fn invoke_from_class(&mut self, klass: Rc<ObjClass>, name: String, arg_count: usize) -> bool {
        let method = match klass.methods.get(&name) {
            Some(method) => method.clone(),
            None => {
                self.runtime_error(format!("Undefined property {}", name));
                return false;
            }
        };
        self.call_value(method, arg_count)
    }
    fn bind_method(&mut self, klass: Rc<ObjClass>, name: String) -> bool {
        let method = match klass.methods.get(&name) {
            Some(method) => method.clone(),
            None => {
                self.runtime_error(format!("Undefined property {}", name));
                return false;
            }
        };

        let receiver = self.peek(0);
        let closure = method.as_closure();
        let bound = ObjBoundMethod {
            receiver,
            method: closure,
        };
        self.pop();
        self.push(Value::Obj(Rc::new(std::cell::RefCell::new(
            Obj::BoundMethod(bound),
        ))));

        true
    }
    fn capture_upvalue(&mut self, local: usize) -> Rc<ObjUpvalue> {
        for upvalue in &self.open_upvalues {
            if upvalue.location == Some(local) {
                return upvalue.clone();
            }
        }
        let new_upvalue = Rc::new(ObjUpvalue {
            location: Some(local),
            closed: Value::Nil,
        });
        self.open_upvalues.push(new_upvalue.clone());
        new_upvalue
    }
}
