use crate::chunks::OpCode;
use crate::object::{
    NativeFn, Obj, ObjBoundMethod, ObjClass, ObjClosure, ObjInstance, ObjNative, ObjString,
    ObjUpvalue,
};
use crate::value::{Value, obj_val};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq)]
pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}
pub struct VM {
    call_frames: Vec<CallFrame>,
    stack: Vec<Value>,
    stack_top: usize,
    globals: HashMap<String, Value>,
    strings: HashMap<String, Value>,
    init_string: ObjString,
    open_upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    bytes_allocated: usize,
    next_gc: usize,
    objects: Vec<Obj>,
    gray_count: usize,
    gray_capacity: usize,
    gray_stack: Vec<Obj>,
}
pub struct CallFrame {
    pub closure: Rc<RefCell<ObjClosure>>,
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
            call_frames: vec![],
            stack: vec![Value::Nil; 1024],
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
            gray_stack: vec![],
        };
        vm.define_native("clock", clock_native);
        vm
    }
    pub fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }
    pub fn pop(&mut self) -> Value {
        if self.stack_top == 0 {
            panic!("Stack underflow");
        }
        self.stack_top -= 1;
        self.stack[self.stack_top].clone()
    }
    fn peek(&mut self, distance: usize) -> Value {
        let index = self.stack_top as isize - 1 - distance as isize;
        if index < 0 {
            println!(
                "DEBUG: peek index out of bounds: distance={} stack_top={} stack_len={}",
                distance,
                self.stack_top,
                self.stack.len()
            );
            panic!("Peek index out of bounds");
        }
        self.stack[index as usize].to_owned()
    }
    fn print_stack(&self) {
        for (i, v) in self.stack.iter().enumerate() {
            if i >= self.stack_top {
                break;
            }
            print!("{i}: ");
            v.print();
            println!();
        }
    }
    fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.stack = vec![Value::Nil; 1024];
        self.open_upvalues = vec![];
    }
    fn runtime_error(&mut self, msg: String) {
        eprintln!("{msg}");

        for frame in self.call_frames.iter().rev() {
            let closure = frame.closure.borrow();
            let function = &closure.function;

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
    fn define_native(&mut self, name: &str, function: NativeFn) {
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
    fn call(&mut self, closure: Rc<RefCell<ObjClosure>>, arg_count: usize) -> bool {
        let _closure = closure.borrow();
        if arg_count != _closure.function.arity {
            self.runtime_error(format!(
                "Expected {} arguments but got {}.",
                _closure.function.arity, arg_count
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
            slot_start: self.stack_top - arg_count - 1,
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
                let instance = ObjInstance::new(klass.clone());
                let instance_val = obj_val(Obj::Instance(instance));

                let slot = self.stack_top - arg_count - 1;
                self.stack[slot] = instance_val.clone();

                let klass_borrow = klass.clone();

                if let Some(init) = klass_borrow.methods.get("init") {
                    return self.call_value(init.clone(), arg_count);
                }

                if arg_count != 0 {
                    self.runtime_error(format!("Expected 0 arguments, got {}.", arg_count));
                    return false;
                }

                true
            }

            Obj::Closure(c) => self.call(Rc::new(RefCell::new(c.clone())), arg_count),

            Obj::Native(native) => {
                let args_start = self.stack_top - arg_count;
                let result = (native.function)(arg_count, &self.stack[args_start..self.stack_top]);

                self.stack_top -= arg_count + 1;
                self.push(result);

                true
            }

            Obj::BoundMethod(bound) => {
                let slot = self.stack_top - arg_count - 1;
                self.stack[slot] = bound.receiver.clone();

                let method =
                    Value::Obj(Rc::new(RefCell::new(Obj::Closure((*bound.method).clone()))));

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
    fn bind_method(&mut self, klass: Rc<RefCell<ObjClass>>, name: String) -> bool {
        let method = match klass.borrow().methods.get(&name) {
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
        self.push(Value::Obj(Rc::new(RefCell::new(Obj::BoundMethod(bound)))));

        true
    }
    fn capture_upvalue(&mut self, local: usize) -> Rc<RefCell<ObjUpvalue>> {
        for upvalue in &self.open_upvalues {
            if upvalue.borrow().location == Some(local) {
                return upvalue.clone();
            }
        }

        let new_upvalue = Rc::new(RefCell::new(ObjUpvalue::new_open(local)));
        self.open_upvalues.push(new_upvalue.clone());
        new_upvalue
    }
    fn close_upvalue(&mut self, stack: &[Value], last_index: usize) {
        for upvalue in &self.open_upvalues {
            let mut uv = upvalue.borrow_mut();

            if let Some(loc) = uv.location {
                if loc >= last_index {
                    uv.close(stack);
                }
            }
        }
        self.open_upvalues.retain(|uv| uv.borrow().is_open());
    }
    fn close_upvalues(&mut self, last: usize) {
        for upvalue in &self.open_upvalues {
            let mut uv = upvalue.borrow_mut();

            if let Some(location) = uv.location {
                if location >= last {
                    uv.closed = self.stack[location].clone();
                    uv.location = None;
                }
            }
        }

        self.open_upvalues
            .retain(|uv| uv.borrow().location.is_some());
    }
    fn define_method(&mut self, name: String) {
        let method = self.peek(0);
        let klass_val = self.peek(1);
        match klass_val {
            Value::Obj(obj) => {
                let mut klass_ref = obj.borrow_mut();
                match &mut *klass_ref {
                    Obj::Class(c) => {
                        let mut new_class = (**c).clone();
                        new_class.methods.insert(name, method.clone());
                        *c = Rc::new(new_class);
                    }
                    _ => {
                        panic!("Expected class")
                    }
                }
            }
            _ => panic!("Expected object"),
        };

        self.pop(); // method
    }
    fn is_falsey(&self, value: Value) -> bool {
        value.is_nil() || (value.is_bool() && !value.as_bool())
    }
    fn concatenate(&mut self) {
        let b = self.peek(0).as_string().as_str().to_owned();
        let a = self.peek(1).as_string().as_str().to_owned();
        let result = format!("{a}{b}");
        let result_obj = ObjString::from_string(result);
        self.pop();
        self.pop();
        self.push(Value::Obj(Rc::new(RefCell::new(Obj::String(result_obj)))));
    }
    fn run(&mut self) -> InterpretResult {
        loop {
            let frame_index = self.call_frames.len() - 1;

            let instruction = {
                let frame = &mut self.call_frames[frame_index];

                let instruction = frame.closure.borrow().function.chunk.code[frame.ip].clone();
                frame.ip += 1;
                instruction
            };

            match instruction {
                OpCode::Constant(index) => {
                    let constant = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].clone()
                    };

                    self.push(constant);
                }

                OpCode::Nil => {
                    self.push(Value::Nil);
                }

                OpCode::True => {
                    self.push(Value::Bool(true));
                }

                OpCode::False => {
                    self.push(Value::Bool(false));
                }

                OpCode::Pop => {
                    self.pop();
                }

                OpCode::GetLocal(slot) => {
                    let value = {
                        let frame = &self.call_frames[frame_index];

                        self.stack[frame.slot_start + slot as usize].clone()
                    };

                    self.push(value);
                }

                OpCode::SetLocal(slot) => {
                    let value = self.peek(0).clone();

                    let index = {
                        let frame = &self.call_frames[frame_index];

                        frame.slot_start + slot as usize
                    };

                    self.stack[index] = value;
                }

                OpCode::DefineGlobal(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };

                    let value = self.peek(0).clone();
                    self.globals.insert(name.data.clone(), value);
                    self.pop();
                }

                OpCode::GetGlobal(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };

                    match self.globals.get(name.as_str()) {
                        Some(value) => {
                            self.push(value.clone());
                        }

                        None => {
                            self.runtime_error(format!("Undefined variable '{}'.", name.as_str()));

                            return InterpretResult::InterpretRuntimeError;
                        }
                    }
                }

                OpCode::SetGlobal(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };

                    if self.globals.contains_key(&name.data) {
                        let value = self.peek(0).clone();

                        self.globals.insert(name.data.clone(), value);
                    } else {
                        self.runtime_error(format!("Undefined variable '{}'.", name.data));

                        return InterpretResult::InterpretRuntimeError;
                    }
                }

                OpCode::GetUpvalue(slot) => {
                    let upvalue = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().upvalues[slot as usize].clone()
                    };

                    let value = {
                        let uv = upvalue.borrow();

                        match uv.location {
                            Some(index) => self.stack[index].clone(),
                            None => uv.closed.clone(),
                        }
                    };

                    self.push(value);
                }

                OpCode::SetUpvalue(slot) => {
                    let value = self.peek(0).clone();
                    let upvalue = {
                        let frame = &self.call_frames[frame_index];
                        frame.closure.borrow().upvalues[slot as usize].clone()
                    };

                    let mut uv = upvalue.borrow_mut();
                    match &mut uv.location {
                        Some(index) => {
                            self.stack[*index] = value;
                        }
                        None => {
                            uv.closed = value;
                        }
                    }
                }

                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();

                    self.push(Value::Bool(a == b));
                }

                OpCode::Greater => {
                    if !self.peek(0).is_number() || !self.peek(1).is_number() {
                        self.runtime_error("Operands must be numbers.".to_string());

                        return InterpretResult::InterpretRuntimeError;
                    }

                    let b = self.pop().as_number();
                    let a = self.pop().as_number();

                    self.push(Value::Bool(a > b));
                }

                OpCode::Less => {
                    if !self.peek(0).is_number() || !self.peek(1).is_number() {
                        self.runtime_error("Operands must be numbers.".to_string());

                        return InterpretResult::InterpretRuntimeError;
                    }

                    let b = self.pop().as_number();
                    let a = self.pop().as_number();

                    self.push(Value::Bool(a < b));
                }

                OpCode::Add => {
                    let b = self.pop();
                    let a = self.pop();

                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.push(Value::Number(a + b));
                        }

                        (Value::Obj(a), Value::Obj(b)) => {
                            let a = a.borrow();
                            let b = b.borrow();

                            match (&*a, &*b) {
                                (Obj::String(a), Obj::String(b)) => {
                                    let result = format!("{}{}", a.data, b.data);

                                    self.push(Value::Obj(Rc::new(RefCell::new(Obj::String(
                                        ObjString::from_string(result),
                                    )))));
                                }

                                _ => {
                                    self.runtime_error(
                                        "Operands must be two numbers or two strings.".to_string(),
                                    );

                                    return InterpretResult::InterpretRuntimeError;
                                }
                            }
                        }

                        _ => {
                            self.runtime_error(
                                "Operands must be two numbers or two strings.".to_string(),
                            );

                            return InterpretResult::InterpretRuntimeError;
                        }
                    }
                }

                OpCode::Subtract => {
                    if !self.peek(0).is_number() || !self.peek(1).is_number() {
                        self.runtime_error("Operands must be numbers.".to_string());

                        return InterpretResult::InterpretRuntimeError;
                    }

                    let b = self.pop().as_number();
                    let a = self.pop().as_number();

                    self.push(Value::Number(a - b));
                }

                OpCode::Multiply => {
                    if !self.peek(0).is_number() || !self.peek(1).is_number() {
                        self.runtime_error("Operands must be numbers.".to_string());

                        return InterpretResult::InterpretRuntimeError;
                    }

                    let b = self.pop().as_number();
                    let a = self.pop().as_number();

                    self.push(Value::Number(a * b));
                }

                OpCode::Divide => {
                    if !self.peek(0).is_number() || !self.peek(1).is_number() {
                        self.runtime_error("Operands must be numbers.".to_string());

                        return InterpretResult::InterpretRuntimeError;
                    }

                    let b = self.pop().as_number();
                    let a = self.pop().as_number();

                    self.push(Value::Number(a / b));
                }

                OpCode::Not => {
                    let value = self.pop();

                    self.push(Value::Bool(self.is_falsey(value)));
                }

                OpCode::Negate => {
                    if !self.peek(0).is_number() {
                        self.runtime_error("Operand must be a number.".to_string());

                        return InterpretResult::InterpretRuntimeError;
                    }

                    let value = self.pop().as_number();

                    self.push(Value::Number(-value));
                }

                OpCode::Print => self.pop().print(),

                OpCode::Jump(offset) => {
                    let frame = &mut self.call_frames[frame_index];

                    frame.ip += offset as usize;
                }

                OpCode::JumpIfFalse(offset) => {
                    let value = self.peek(0);
                    if self.is_falsey(value) {
                        let frame = &mut self.call_frames[frame_index];

                        frame.ip += offset as usize;
                    }
                }

                OpCode::Loop(offset) => {
                    let frame = &mut self.call_frames[frame_index];

                    frame.ip -= offset as usize;
                }

                OpCode::Call(arg_count) => {
                    let callee = self.peek(arg_count as usize).clone();

                    if !self.call_value(callee, arg_count as usize) {
                        return InterpretResult::InterpretRuntimeError;
                    }
                }

                OpCode::Closure(index) => {
                    let function = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize]
                            .as_function()
                    };

                    let mut closure = ObjClosure::new(function);

                    for _ in 0..closure.function.upvalue_count {
                        let is_local;
                        let upvalue_index;
                        {
                            let frame = &mut self.call_frames[frame_index];
                            is_local = match frame.closure.borrow().function.chunk.code[frame.ip] {
                                OpCode::Data(d) => d == 1,
                                _ => panic!("Expected Data opcode for upvalue is_local"),
                            };
                            frame.ip += 1;
                            upvalue_index =
                                match frame.closure.borrow().function.chunk.code[frame.ip] {
                                    OpCode::Data(d) => d,
                                    _ => panic!("Expected Data opcode for upvalue index"),
                                };
                            frame.ip += 1;
                        }

                        if is_local {
                            let slot_start = self.call_frames[frame_index].slot_start;
                            closure
                                .upvalues
                                .push(self.capture_upvalue(slot_start + upvalue_index as usize));
                        } else {
                            closure.upvalues.push(Rc::clone(
                                &self.call_frames[frame_index].closure.borrow().upvalues
                                    [upvalue_index as usize],
                            ));
                        }
                    }

                    self.push(Value::Obj(Rc::new(RefCell::new(Obj::Closure(closure)))));
                }

                OpCode::CloseUpvalue => {
                    let last = self.stack_top - 1;

                    self.close_upvalues(last);

                    self.pop();
                }

                OpCode::Return => {
                    let result = self.pop();
                    let frame = self.call_frames.pop().unwrap();

                    self.close_upvalues(frame.slot_start);

                    if self.call_frames.is_empty() {
                        return InterpretResult::InterpretOk;
                    }
                    self.stack_top = frame.slot_start;
                    self.push(result);
                }

                OpCode::Class(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };
                    let class = ObjClass {
                        name: name.data.clone(),
                        methods: HashMap::new(),
                    };

                    self.push(Value::Obj(Rc::new(RefCell::new(Obj::Class(Rc::new(
                        class,
                    ))))));
                }
                OpCode::Method(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };
                    self.define_method(name.data.to_owned());
                }
                OpCode::SetProperty(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];
                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };

                    let receiver = self.peek(1).clone();
                    let value = self.peek(0).clone();

                    match receiver {
                        Value::Obj(obj) => {
                            let mut obj_ref = obj.borrow_mut();

                            match &mut *obj_ref {
                                Obj::Instance(instance) => {
                                    instance.fields.insert(name.data.clone(), value.clone());

                                    self.pop(); // value
                                    self.pop(); // receiver
                                }

                                _ => {
                                    self.runtime_error("Only instances have fields.".to_string());
                                    return InterpretResult::InterpretRuntimeError;
                                }
                            }
                        }

                        _ => {
                            self.runtime_error("Only instances have fields.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    }
                }
                OpCode::GetProperty(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];
                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };

                    let receiver = self.peek(0).clone();

                    let result = match &receiver {
                        Value::Obj(obj) => match &*obj.borrow() {
                            Obj::Instance(instance) => {
                                if let Some(value) = instance.fields.get(&name.data) {
                                    value.clone()
                                } else if let Some(method) = instance.klass.methods.get(&name.data)
                                {
                                    let bound = ObjBoundMethod {
                                        receiver: receiver.clone(),
                                        method: method.as_closure(),
                                    };

                                    Value::Obj(Rc::new(RefCell::new(Obj::BoundMethod(bound))))
                                } else {
                                    self.runtime_error(format!(
                                        "Undefined property '{}'.",
                                        name.data
                                    ));
                                    return InterpretResult::InterpretRuntimeError;
                                }
                            }
                            _ => {
                                self.runtime_error("Only instances have properties.".to_string());
                                return InterpretResult::InterpretRuntimeError;
                            }
                        },
                        _ => {
                            self.runtime_error("Only instances have properties.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    self.pop();
                    self.push(result);
                }
                OpCode::Invoke(index, arg_count) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };
                    let receiver = self.peek(arg_count as usize).clone();

                    match receiver {
                        Value::Obj(obj) => {
                            let obj_ref = obj.borrow();

                            match &*obj_ref {
                                Obj::Instance(instance) => {
                                    if let Some(value) = instance.fields.get(&name.data) {
                                        let value = value.clone();

                                        let slot = self.stack_top - 1 - arg_count as usize;
                                        self.stack[slot] = value.clone();

                                        if !self.call_value(value, arg_count as usize) {
                                            return InterpretResult::InterpretRuntimeError;
                                        }

                                        continue;
                                    }

                                    if !self.invoke_from_class(
                                        instance.klass.clone(),
                                        name.data.clone(),
                                        arg_count as usize,
                                    ) {
                                        return InterpretResult::InterpretRuntimeError;
                                    }
                                }

                                _ => {
                                    self.runtime_error("Only instances have methods.".to_string());

                                    return InterpretResult::InterpretRuntimeError;
                                }
                            }
                        }

                        _ => {
                            self.runtime_error("Only instances have methods.".to_string());

                            return InterpretResult::InterpretRuntimeError;
                        }
                    }
                }
                OpCode::Inherit => {
                    let superclass_val = self.peek(1).clone();
                    let subclass_val = self.peek(0).clone();

                    let super_class = match superclass_val {
                        Value::Obj(ref obj) => match &*obj.borrow() {
                            Obj::Class(c) => c.clone(),
                            _ => {
                                self.runtime_error(
                                    "Inherit: Superclass must be a class.".to_string(),
                                );
                                return InterpretResult::InterpretRuntimeError;
                            }
                        },
                        _ => {
                            self.runtime_error("Inherit: Superclass must be a class.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    let sub_obj = match subclass_val {
                        Value::Obj(ref obj) => Rc::clone(obj),
                        _ => {
                            self.runtime_error("Subclass must be a class.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    {
                        let mut tmp_obj = sub_obj.borrow_mut();
                        let sub_class = match &*tmp_obj {
                            Obj::Class(c) => Rc::clone(c),
                            _ => {
                                self.runtime_error("Subclass must be a class.".to_string());
                                return InterpretResult::InterpretRuntimeError;
                            }
                        };

                        let mut new_methods = sub_class.methods.clone();

                        for (name, method) in &super_class.methods {
                            new_methods.insert(name.clone(), method.clone());
                        }

                        let new_sub_class = Rc::new(ObjClass {
                            name: sub_class.name.clone(),
                            methods: new_methods,
                        });

                        *tmp_obj = Obj::Class(new_sub_class);
                    }

                    self.push(subclass_val);
                }
                OpCode::GetSuper(index) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };

                    // stack:
                    // ... receiver superclass
                    let superclass = self.pop();
                    let superclass = match superclass {
                        Value::Obj(obj) => match &*obj.borrow() {
                            Obj::Class(class) => class.clone(),
                            _ => {
                                self.runtime_error(
                                    "GetSuper: Superclass must be a class.".to_string(),
                                );
                                return InterpretResult::InterpretRuntimeError;
                            }
                        },

                        _ => {
                            self.runtime_error("GetSuper: Superclass must be a class.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    let receiver = self.peek(0).clone();
                    let method = match superclass.methods.get(&name.data) {
                        Some(method) => method.clone(),
                        None => {
                            self.runtime_error(format!("Undefined property '{}'.", name.data));

                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    let bound_method = ObjBoundMethod {
                        receiver,
                        method: method.as_closure(),
                    };

                    self.pop();
                    self.push(Value::Obj(Rc::new(RefCell::new(Obj::BoundMethod(
                        bound_method,
                    )))));
                }
                OpCode::SuperInvoke(index, arg_count) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };
                    let superclass = self.pop();
                    let superclass = match superclass {
                        Value::Obj(obj) => match &*obj.borrow() {
                            Obj::Class(class) => class.clone(),
                            _ => {
                                self.runtime_error(
                                    "SuperInvoke: Superclass must be a class.".to_string(),
                                );
                                return InterpretResult::InterpretRuntimeError;
                            }
                        },

                        _ => {
                            self.runtime_error(
                                "SuperInvoke: Superclass must be a class.".to_string(),
                            );
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    if !self.invoke_from_class(superclass, name.data.clone(), arg_count as usize) {
                        return InterpretResult::InterpretRuntimeError;
                    }
                }

                OpCode::Data(_) => {
                    // Data opcodes are consumed by other opcodes like Closure
                    // If we hit one here, it means something is wrong with the bytecode stream
                    panic!("Unexpected Data opcode in main loop");
                }
                OpCode::Nop => {}

                _ => {
                    println!("Unhandled opcode: {:?}", instruction);
                    todo!("Opcode not implemented");
                }
            }
        }
    }
    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler =
            crate::compiler::Compiler::new(None, crate::compiler::FunctionType::TypeScript);
        let function = compiler.compile(source);
        if compiler.had_error {
            return InterpretResult::InterpretCompileError;
        }

        let function_rc = Rc::new(function);
        let closure = Rc::new(RefCell::new(ObjClosure::new(function_rc.clone())));
        // Standard Lox: push closure first, then call.
        // The script closure stays at stack[0] during the entire execution.
        self.push(Value::Obj(Rc::new(RefCell::new(Obj::Closure(
            (*closure.borrow()).clone(),
        )))));

        self.call(closure, 0);
        let result = self.run();
        self.reset_stack();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let mut vm = VM::new();
        let source = "1 + 2 * 3;".to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_arithmetic2() {
        let mut vm = VM::new();
        let source = "1 + 2 + 3;".to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_variables() {
        let mut vm = VM::new();
        let source = r#"
            var a = 1;
            var b = 2;
            var c = a + b;
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_variables2() {
        let mut vm = VM::new();
        let source = r#"
            var a = 1;
            var b = 2;
            var c = a + b;
            c = c + a + b;
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_control_flow() {
        let mut vm = VM::new();
        let source = r#"
            var a = 0;
            if (true) { a = 1; } else { a = 2; }
            var b = 0;
            while (b < 3) { b = b + 1; }
            var c = 0;
            for (var i = 0; i < 3; i = i + 1) { c = c + 1; }
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_closures() {
        let mut vm = VM::new();
        let source = r#"
            fun makeCounter() {
              var i = 0;
              fun count() {
                i = i + 1;
                return i;
              }
              return count;
            }
            var counter = makeCounter();
            counter();
            counter();
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_classes() {
        let mut vm = VM::new();
        let source = r#"
            class Cake {
              eat() {
                return "Eating cake";
              }
            }
            var cake = Cake();
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_class_basic() {
        let mut vm = VM::new();
        let source = r#"
            class Cake {}
            var cake = Cake();
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }

    #[test]
    fn test_inheritance() {
        let mut vm = VM::new();
        let source = r#"
            class superClass {
                doStaff() {
                   print "Inheritance works";
                }
            }
            class subClass < superClass {
                doStaff() {
                   super.doStaff();
                }
            }
            var obj = subClass();
            obj.doStaff();
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_plus_equal() {
        let mut vm = VM::new();
        let source = r#"
           var x = 10;
           x += 2;
           print x;
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_recursion() {
        let mut vm = VM::new();
        let source = r#"
        fun test(i) {
            if (i > 0) {
                test(i - 1);
                print i;
              }
        }
        test(4);
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class_methods() {
        let mut vm = VM::new();
        let source = r#"
        class TestClass {
                class doStaff() {
                   print "class method";
            }
        }
        TestClass.doStaff();
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class1() {
        let mut vm = VM::new();
        let source = r#"
        class TestClass {
            doStaff() {
                print "doing staff";
                return "Done";
            }
        }
        print TestClass;
        var testObject = TestClass();
        testObject.test = "test";
        print testObject;
        print testObject.test;
        testObject.doStaff();
        print testObject.doStaff();
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class2() {
        let mut vm = VM::new();
        let source = r#"
        class TestClass {
                init(x) {
                  this.test = x;
                }
                doStaff(y) {
                   print this.test + y;
            }
        }
        var obj = TestClass(2);
        obj.doStaff(4);
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class3() {
        let mut vm = VM::new();
        let source = r#"
        class TestClass {
                init(x) {
                  print("in init");
                  this.x = x;
                }
                doStaff() {
                   print this.x;
            }
        }
        var obj = TestClass("test");
        obj.doStaff();
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class4() {
        let mut vm = VM::new();
        let source = r#"class TestClass {
                                init(x) {
                                  this.test = x;
                                }
                                doStaff(y) {
                                  return this.test + y;
                                }
                            }
                          var obj = TestClass(2);
                          var sum = 0;
                          while (sum < 1000) {
                            sum = sum + obj.doStaff(4) + obj.doStaff(4);
                          }
                          print sum;"#
            .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class5() {
        let mut vm = VM::new();
        let source = r#"class TestClass {
                                init() {
                                  this.test = 5;
                                }
                                doStaff() {
                                  return this.test;
                                }
                            }
                          var obj = TestClass();
                          var sum = 0;
                          while (sum < 1000) {
                            sum = sum + obj.doStaff() + obj.doStaff();
                          }
                          print sum;"#
            .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class6() {
        let mut vm = VM::new();
        let source = r#"class TestClass {
                                init() {
                                  this.test1 = 5;
                                  this.test2 = 4;
                                }
                                doStaff1() {
                                  return this.test1;
                                }
                                doStaff2() {
                                  return this.test2;
                                }
                            }
                          var o = TestClass();
                          var sum = 0;
                          while (sum < 1000) {
                            sum = sum + o.doStaff1() + o.doStaff2();
                          }
                          print sum;"#
            .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_class7() {
        let mut vm = VM::new();
        let source = r#"class TestClass {
                            init() {
                              this.test1 = 1;
                              this.test2 = 2;
                              this.test3 = 2;
                            }
                            staff1() { return this.test1; }
                            staff2() { return this.test2; }
                            staff3() { return this.test3; }
                        }
                      var obj = TestClass();
                      var sum = 0;
                      while (sum < 1000) {
                        sum = sum + obj.staff1() + obj.staff2() + obj.staff3();
                      }
                      print sum;"#
            .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_native() {
        let mut vm = VM::new();
        let source = r#"
        var start = clock();
        var sum = 0;
        while (sum < 1000) {
            sum = sum + 1;
        }
        print clock() - start;
        print sum;
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_native2() {
        let mut vm = VM::new();
        let source = r#"class TestClass {
                                init(x) {
                                  this.test=x;
                                }
                                doStaff(y) {
                                  return this.test + y;
                                }
                            }
                          var obj = TestClass(2);
                          var sum = 0;
                          var start = clock();
                          while (sum < 1000) {
                            sum = sum + obj.doStaff(4);
                          }
                          print clock() - start;
                          print sum;"#
            .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_benchmark() {
        let mut vm = VM::new();
        let source = r#"
        class zoo {
            init() {
                this.aardvark = 1;
                this.baboon = 1;
                this.cat = 1;
                // this.donkey = 1;
                // this.elephant = 1;
                // this.fox = 1;
            }
            ant() { return this.aardvark; }
            banana() { return this.baboon; }
            tuna() { return this.cat; }
            hay() { return this.donkey; }
            grass() { return this.elephant; }
            moose() { return this.fox; }
        }

        var z = zoo();
        var sum = 0;
       var start = clock();
        while (sum < 1099900) {
            sum = sum + z.ant() + z.banana() + z.tuna() + z.hay() + z.grass() + z.moose();
        }
        print clock() - start;
        print sum;
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_fib() {
        let mut vm = VM::new();
        let source = r#"
        fun fib(n) {
          if (n <= 1) return n;
          return fib(n - 2) + fib(n - 1);
        }

        for (var i = 0; i < 26; i = i + 1) {
          print fib(i);
        }
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
    #[test]
    fn test_while() {
        let mut vm = VM::new();
        let source = r#"
        {
            var i = 0;
            while (i < 10) {
                print i;
                i = i +1;
            }
        }
        "#
        .to_string();
        assert_eq!(vm.interpret(source), InterpretResult::InterpretOk);
    }
}
