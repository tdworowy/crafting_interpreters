use crate::object::{
    NativeFn, Obj, ObjClass, ObjClosure, ObjInstance, ObjNative, ObjString, ObjType, ObjUpvalue,
};
use crate::value::{Value, obj_val};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct VM {
    call_frames: Vec<CallFrame>,
    stack: Vec<Value>,
    stack_top: usize,
    globals: HashMap<String, Value>,
    strings: HashMap<String, Value>,
    init_string: ObjString,
    open_upvalue: Option<ObjUpvalue>,
    bytes_allocated: usize,
    next_gc: usize,
    objects: Vec<Obj>,
    gray_count: usize,
    gray_capacity: usize,
    gray_stack: Vec<Obj>,
}
struct CallFrame {
    ip: usize,
    closure: ObjClosure,
    slots: Vec<Value>,
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
            open_upvalue: None,
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
        self.stack[self.stack_top as usize - distance as usize].to_owned()
    }
    fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.stack = vec![];
        self.open_upvalue = None;
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
        self.push(obj_val(
            Box::into_raw(Box::new(ObjString::copy_from_str(name))) as *mut Obj,
        ));

        self.push(obj_val(
            Box::into_raw(Box::new(ObjNative::new(function))) as *mut Obj
        ));

        let s = self.stack[0].as_string();
        let key = unsafe { &(*s).data };
        let value = self.stack[1].clone();

        self.globals.entry(key.to_owned()).or_insert(value);

        self.pop();
        self.pop();
    }
    fn call(&mut self, obj_closure: ObjClosure, arg_count: usize) -> bool {
        if arg_count != obj_closure.function.arity {
            self.runtime_error(format!(
                "Expected {} arguments but got {}.",
                obj_closure.function.arity, arg_count
            ));
            false
        } else {
            let mut frame: CallFrame = self.call_frames.pop().unwrap();
            frame.closure = obj_closure.clone();
            frame.ip = obj_closure.function.chunk.code.len(); // not sure if it works
            frame
                .slots
                .push(self.stack[self.stack_top as usize].to_owned());
            true
        }
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> bool {
        if !callee.is_obj() {
            self.runtime_error("Can only call functions and classes.".to_owned());
            return false;
        }

        unsafe {
            match (*callee.as_obj()).obj_type {
                ObjType::ObjClass => {
                    let klass = callee.as_class();

                    // Create instance
                    let instance = ObjInstance::new((*klass).clone(), HashMap::new());
                    let instance_ptr = Box::into_raw(Box::new(instance)) as *mut Obj;

                    // Replace callee slot with instance
                    self.stack[self.stack_top - arg_count - 1] = obj_val(instance_ptr);

                    // Call initializer if exists
                    if let Some(init) = (*klass).methods.get("init") {
                        self.call(init.as_closure(), arg_count)
                    } else {
                        if arg_count != 0 {
                            self.runtime_error(format!("Expected 0 arguments, got {}.", arg_count));
                            return false;
                        }
                        true
                    }
                }

                ObjType::ObjClosure => self.call(*callee.as_closure(), arg_count),

                ObjType::ObjNative => {
                    let native = callee.as_native();

                    let args_start = self.stack_top - arg_count;
                    let result =
                        ((*native).function)(arg_count, &self.stack[args_start..self.stack_top]);

                    // Pop args + callee, push result
                    self.stack_top -= arg_count + 1;
                    self.push(result);

                    true
                }

                ObjType::ObjBoundMethod => {
                    let bound = callee.as_bound_method();

                    // Replace callee with receiver
                    self.stack[self.stack_top - arg_count - 1] = (*bound).receiver.clone();
                    let method = (*bound).method.clone();
                    self.call(method, arg_count)
                }

                _ => {
                    self.runtime_error("Can only call functions and classes.".to_owned());
                    false
                }
            }
        }
    }
}
