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
        self.stack[self.stack_top - 1 - distance as usize].to_owned()
    }
    fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.stack = vec![];
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
                let instance = Obj::Instance(ObjInstance::new(klass.clone()));

                let instance_val = obj_val(instance);

                let slot = self.stack_top - arg_count - 1;
                self.stack[slot] = instance_val.clone();

                let klass_borrow = klass.borrow();

                if let Some(init) = klass_borrow.methods.get("init") {
                    //drop(obj); // safe before recursive call
                    self.call_value(init.clone(), arg_count)
                } else {
                    if arg_count != 0 {
                        self.runtime_error(format!("Expected 0 arguments, got {}.", arg_count));
                        return false;
                    }
                    true
                }
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
    fn invoke_from_class(
        &mut self,
        klass: Rc<RefCell<ObjClass>>,
        name: String,
        arg_count: usize,
    ) -> bool {
        let method = match klass.borrow().methods.get(&name) {
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
        let method: Value = self.peek(0);
        let klass = self.peek(1).as_class();
        klass.borrow_mut().methods.insert(name, method.clone());
        self.pop();
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
                    let callee = self.peek(arg_count as i64).clone();

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

                    let closure = ObjClosure::new(function);

                    self.push(Value::Obj(Rc::new(RefCell::new(Obj::Closure(closure)))));
                }

                OpCode::CloseUpvalue => {
                    let last = self.stack.len() - 1;

                    self.close_upvalues(last);

                    self.pop();
                }

                OpCode::Return => {
                    let result = self.pop();
                    let frame = self.call_frames.pop().unwrap();

                    self.close_upvalues(frame.slot_start);

                    if self.call_frames.is_empty() {
                        self.pop();

                        return InterpretResult::InterpretOk;
                    }

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
                        RefCell::new(class),
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

                    match receiver {
                        Value::Obj(obj) => {
                            let mut obj_ref = obj.borrow_mut();

                            match &mut *obj_ref {
                                Obj::Instance(instance) => {
                                    let value = self.peek(0).clone();

                                    instance.fields.insert(name.data.clone(), value.clone());
                                    let value = self.pop();
                                    self.pop();
                                    self.push(value);
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

                    match &receiver {
                        Value::Obj(obj) => {
                            let obj_ref = obj.borrow();

                            match &*obj_ref {
                                Obj::Instance(instance) => {
                                    // Fields first.
                                    if let Some(value) = instance.fields.get(&name.data) {
                                        let value = value.clone();

                                        self.pop();
                                        self.push(value);
                                    } else {
                                        // Then methods.
                                        let method =
                                            match instance.klass.borrow().methods.get(&name.data) {
                                                Some(method) => method.clone(),
                                                None => {
                                                    self.runtime_error(format!(
                                                        "Undefined property '{}'.",
                                                        name.data
                                                    ));

                                                    return InterpretResult::InterpretRuntimeError;
                                                }
                                            };

                                        let bound_method = ObjBoundMethod {
                                            receiver: receiver.clone(),
                                            method: method.as_closure(),
                                        };

                                        self.pop();

                                        self.push(Value::Obj(Rc::new(RefCell::new(
                                            Obj::BoundMethod(bound_method),
                                        ))));
                                    }
                                }

                                _ => {
                                    self.runtime_error(
                                        "Only instances have properties.".to_string(),
                                    );

                                    return InterpretResult::InterpretRuntimeError;
                                }
                            }
                        }

                        _ => {
                            self.runtime_error("Only instances have properties.".to_string());

                            return InterpretResult::InterpretRuntimeError;
                        }
                    }
                }
                OpCode::Invoke(index, arg_count) => {
                    let name = {
                        let frame = &self.call_frames[frame_index];

                        frame.closure.borrow().function.chunk.constants[index as usize].as_string()
                    };
                    let receiver = self.peek(arg_count as i64).clone();

                    match receiver {
                        Value::Obj(obj) => {
                            let obj_ref = obj.borrow();

                            match &*obj_ref {
                                Obj::Instance(instance) => {
                                    if let Some(value) = instance.fields.get(&name.data) {
                                        let value = value.clone();

                                        let slot = self.stack.len() - 1 - arg_count as usize;
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
                    let superclass = self.peek(1).clone();
                    let subclass = self.peek(0).clone();

                    let super_class = match superclass {
                        Value::Obj(obj) => match &*obj.borrow() {
                            Obj::Class(c) => c.clone(),
                            _ => {
                                self.runtime_error("Superclass must be a class.".to_string());
                                return InterpretResult::InterpretRuntimeError;
                            }
                        },
                        _ => {
                            self.runtime_error("Superclass must be a class.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    let sub_obj = match subclass {
                        Value::Obj(obj) => obj,
                        _ => {
                            self.runtime_error("Subclass must be a class.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };
                    let mut tmp_obj = sub_obj.borrow_mut();
                    let mut sub_class = match &mut *tmp_obj {
                        Obj::Class(c) => c.borrow_mut(),
                        _ => {
                            self.runtime_error("Subclass must be a class.".to_string());
                            return InterpretResult::InterpretRuntimeError;
                        }
                    };

                    for (name, method) in &super_class.borrow().methods {
                        sub_class.methods.insert(name.clone(), method.clone());
                    }

                    self.pop(); // subclass
                    self.pop(); // superclass
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
        self.push(Value::Obj(Rc::new(RefCell::new(Obj::Function(
            function.clone(),
        )))));
        let closure = ObjClosure::new(Rc::new(function));
        self.pop();
        self.push(Value::Obj(Rc::new(RefCell::new(Obj::Closure(
            closure.clone(),
        )))));

        self.call(Rc::new(RefCell::new(closure)), 0);
        self.run()
    }
}

// TODO handle GetSupper and SuperInvoke
