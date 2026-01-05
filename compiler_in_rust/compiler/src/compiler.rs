use std::collections::HashMap;

use crate::{
    chunks::{Chunk, OpCode},
    object::ObjFunction,
    scaner::{Scanner, Token, TokenType},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Precedence {
    PREC_NONE,
    PREC_ASSIGNMENT,
    PREC_OR,
    PREC_AND,
    PREC_EQUALITY,
    PREC_COMPARISON,
    PREC_TERM,
    PREC_FACTOR,
    PREC_UNARY,
    PREC_CALL,
    PREC_PRIMARY,
}

#[derive(Copy, Clone, Debug)]
enum ExprsionType {
    GROUPING,
    CALL,
    BINARY,
    UNARY,
    DOT,
    VARIABLE,
    STRING,
    NUMBER,
    LITERAL,
    SUPER,
    THIS,
    OR,
    AND,
}

type ParseFn = fn(can_assign: bool);

#[derive(Copy, Clone, Debug)]
struct ParseRule {
    pub prefix: Option<ExprsionType>,
    pub infix: Option<ExprsionType>,
    pub precedence: Precedence,
}

lazy_static::lazy_static! {
    static ref RULES: HashMap<TokenType, ParseRule> = {
        let mut m = HashMap::new();

        m.insert(TokenType::TOKEN_LEFT_PAREN,     ParseRule { prefix: Some(ExprsionType::GROUPING), infix: Some(ExprsionType::CALL),    precedence: Precedence::PREC_CALL });
        m.insert(TokenType::TOKEN_RIGHT_PAREN,    ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_LEFT_BRACE,     ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_RIGHT_BRACE,    ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_DOT,            ParseRule { prefix: None,                         infix: Some(ExprsionType::DOT),     precedence: Precedence::PREC_CALL });
        m.insert(TokenType::TOKEN_MINUS,          ParseRule { prefix: Some(ExprsionType::UNARY),    infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_TERM });
        m.insert(TokenType::TOKEN_PLUS,           ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_TERM });
        m.insert(TokenType::TOKEN_SEMICOLON,      ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_SLASH,          ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_FACTOR });
        m.insert(TokenType::TOKEN_STAR,           ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_FACTOR });
        m.insert(TokenType::TOKEN_BANG,           ParseRule { prefix: Some(ExprsionType::UNARY),    infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_BANG_EQUAL,     ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_EQUALITY });
        m.insert(TokenType::TOKEN_EQUAL,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_EQUAL_EQUAL,    ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_EQUALITY });
        m.insert(TokenType::TOKEN_GREATER,        ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_GREATER_EQUAL,  ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_LESS,           ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_LESS_EQUAL,     ParseRule { prefix: None,                         infix: Some(ExprsionType::BINARY),  precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_IDENTIFIER,     ParseRule { prefix: Some(ExprsionType::VARIABLE), infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_STRING,         ParseRule { prefix: Some(ExprsionType::STRING),   infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_NUMBER,         ParseRule { prefix: Some(ExprsionType::NUMBER),   infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_AND,            ParseRule { prefix: None,                         infix: Some(ExprsionType::AND),     precedence: Precedence::PREC_AND });
        m.insert(TokenType::TOKEN_CLASS,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_ELSE,           ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_FALSE,          ParseRule { prefix: Some(ExprsionType::LITERAL),  infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_FOR,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_FUN,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_IF,             ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_NIL,            ParseRule { prefix: Some(ExprsionType::LITERAL),  infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_OR,             ParseRule { prefix: None,                         infix: Some(ExprsionType::OR),      precedence: Precedence::PREC_OR });
        m.insert(TokenType::TOKEN_PRINT,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_RETURN,         ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_SUPER,          ParseRule { prefix: Some(ExprsionType::SUPER),    infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_THIS,           ParseRule { prefix: Some(ExprsionType::THIS),     infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_TRUE,           ParseRule { prefix: Some(ExprsionType::LITERAL),  infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_VAR,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_WHILE,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_ERROR,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_EOF,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PREC_NONE });

        m
    };
}
fn get_rule(token_type: TokenType) -> ParseRule {
    RULES.get(&token_type).copied().unwrap_or(ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::PREC_NONE,
    })
}
#[derive(Clone)]
struct Local {
    name: Token,
    depth: isize,
    is_captured: bool,
}

#[derive(Clone)]
struct Upvalue {
    index: isize,
    is_local: bool,
}

#[derive(PartialEq, Clone)]
enum FunctionType {
    TYPE_FUNCTION,
    TYPE_SCRIPT,
    TYPE_METHOD,
    TYPE_INITIALIZER,
}

#[derive(Clone)]
struct Compiler {
    enclosing: Option<Box<Compiler>>,
    class_compiler: Option<Box<ClassCompiler>>,
    function: Box<ObjFunction>,
    function_type: FunctionType,
    locals: Vec<Local>,
    upvalues: Vec<Upvalue>,
    scope_depth: isize,
    scanner: Option<Scanner>,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
} // I have combited Parser and Compiler, it may have some side effects 

#[derive(Clone)]
struct ClassCompiler {
    enclosing: Option<Box<ClassCompiler>>,
    has_super_class: bool,
}

impl Precedence {
    pub fn next(&self) -> Precedence {
        match self {
            Precedence::PREC_NONE => Precedence::PREC_ASSIGNMENT,
            Precedence::PREC_ASSIGNMENT => Precedence::PREC_OR,
            Precedence::PREC_OR => Precedence::PREC_AND,
            Precedence::PREC_AND => Precedence::PREC_EQUALITY,
            Precedence::PREC_EQUALITY => Precedence::PREC_COMPARISON,
            Precedence::PREC_COMPARISON => Precedence::PREC_TERM,
            Precedence::PREC_TERM => Precedence::PREC_FACTOR,
            Precedence::PREC_FACTOR => Precedence::PREC_UNARY,
            Precedence::PREC_UNARY => Precedence::PREC_CALL,
            Precedence::PREC_CALL => Precedence::PREC_PRIMARY,
            Precedence::PREC_PRIMARY => Precedence::PREC_PRIMARY, // highest
        }
    }
}

impl Compiler {
    fn new(enclosing: Option<Box<Compiler>>, function_type: FunctionType) -> Self {
        let mut compiler = Compiler {
            enclosing: enclosing.clone(),
            function_type: function_type.clone(),
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            function: Box::new(ObjFunction::new()),
            scanner: None,
            current: Token {
                token_type: TokenType::TOKEN_SYNTHETIC,
                lexeme: "".to_owned(),
                line: 0,
            },
            previous: Token {
                token_type: TokenType::TOKEN_SYNTHETIC,
                lexeme: "".to_owned(),
                line: 0,
            },
            had_error: false,
            panic_mode: false,
            class_compiler: None,
        };

        if function_type != FunctionType::TYPE_SCRIPT {
            compiler.function.name = enclosing.unwrap().previous.lexeme.clone();
        }
        let local_name = if function_type != FunctionType::TYPE_FUNCTION {
            Token {
                token_type: TokenType::TOKEN_THIS,
                lexeme: "this".to_owned(),
                line: 0,
            }
        } else {
            Token {
                token_type: TokenType::TOKEN_SYNTHETIC,
                lexeme: "".to_owned(),
                line: 0,
            }
        };

        compiler.locals.push(Local {
            name: local_name,
            depth: 0,
            is_captured: false,
        });

        compiler
    }
    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn emit_byte(&mut self, byte: OpCode) {
        let line = self.previous.line;
        self.current_chunk().write_chunk(byte, line);
    }
    fn emit_bytes(&mut self, byte1: OpCode, byte2: OpCode) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }
    fn emit_jump(&mut self, instruction: fn(i16) -> OpCode) -> usize {
        self.emit_byte(instruction(0));
        self.current_chunk().code.len() - 1
    }
    fn patch_jump(&mut self, jump_index: usize) {
        let offset = self.current_chunk().code.len() - jump_index - 1;

        let opcode = &mut self.current_chunk().code[jump_index];

        match opcode {
            OpCode::Jump(o) | OpCode::JumpIfFalse(o) | OpCode::Loop(o) => {
                *o = offset as i16;
            }
            _ => panic!("Invalid jump patch"),
        }
    }
    fn emit_constant(&mut self, value: String) {
        let constant_index = self.current_chunk().add_constant(value);
        self.emit_byte(OpCode::Constant(constant_index));
    }
    fn emit_return(&mut self) {
        if self.function_type == FunctionType::TYPE_INITIALIZER {
            self.emit_bytes(OpCode::OP_GET_LOCAL, OpCode::Constant(0));
        } else {
            self.emit_byte(OpCode::OP_NIL);
        }
        self.emit_byte(OpCode::OP_RETURN);
    }
    fn error_et(&mut self, token: Token, message: String) {
        if self.panic_mode {
            return;
        }
        eprint!("[line {}]", token.line);
        match token.token_type {
            TokenType::TOKEN_EOF => {
                eprint!(" at end");
            }
            TokenType::TOKEN_ERROR => {}
            _ => {
                eprint!(" at {}", token.lexeme);
            }
        }
        eprintln!(":{}", message);
        self.had_error = true;
    }

    fn error_at_current(&mut self, message: String) {
        self.error_et(self.current.clone(), message);
    }
    fn error(&mut self, message: String) {
        self.error_et(self.previous.clone(), message);
    }
    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.as_mut().unwrap().scan_token();
            if (self.current.token_type != TokenType::TOKEN_ERROR) {
                break;
            } else {
                self.error_at_current(self.current.lexeme.clone());
            }
        }
    }
    fn consume(&mut self, token_type: TokenType, message: String) {
        if self.current.token_type == token_type {
            self.advance();
        } else {
            self.error_at_current(message);
        }
    }
    fn check(&self, token_type: TokenType) -> bool {
        self.current.token_type == token_type
    }
    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        } else {
            self.advance();
            return true;
        }
    }
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let prefix_rule = match get_rule(self.previous.token_type.clone()).prefix {
            Some(rule) => rule,
            _ => {
                self.error("Expect expression.".to_owned());
                return;
            }
        };
        let can_assign = precedence <= Precedence::PREC_ASSIGNMENT;
        match prefix_rule {
            ExprsionType::GROUPING => self.groping(can_assign),
            ExprsionType::UNARY => self.unary(can_assign),
            ExprsionType::VARIABLE => self.variable(can_assign),
            ExprsionType::STRING => self.string(can_assign),
            ExprsionType::NUMBER => self.number(can_assign),
            ExprsionType::LITERAL => self.literal(can_assign),
            ExprsionType::SUPER => self.super_(can_assign),
            ExprsionType::THIS => self.this(can_assign),
            _ => self.error("Incorect prefix rule".to_owned()),
        }
        while precedence <= get_rule(self.current.token_type.clone()).precedence {
            self.advance();
            let infix_rule = match get_rule(self.previous.token_type.clone()).infix {
                Some(rule) => rule,
                _ => {
                    return;
                }
            };
            match infix_rule {
                ExprsionType::CALL => self.call(can_assign),
                ExprsionType::DOT => self.dot(can_assign),
                ExprsionType::AND => self.and(can_assign),
                ExprsionType::OR => self.or(can_assign),
                ExprsionType::BINARY => self.binary(can_assign),
                _ => self.error("Incorect infix rule".to_owned()),
            }
            if can_assign && self.match_token(TokenType::TOKEN_EQUAL) {
                self.error("Invalid assigment target.".to_owned());
            }
        }
    }
    fn string(&mut self, can_assign: bool) {
        let value = self.previous.lexeme.clone().replace("\"", "");
        self.emit_constant(value);
    }
    fn number(&mut self, can_assign: bool) {
        self.emit_constant(self.previous.lexeme.clone());
    }
    fn literal(&mut self, can_assign: bool) {
        match self.previous.token_type {
            TokenType::TOKEN_FALSE => self.emit_byte(OpCode::OP_FALSE),
            TokenType::TOKEN_TRUE => self.emit_byte(OpCode::OP_TRUE),
            TokenType::TOKEN_NIL => self.emit_byte(OpCode::OP_NIL),
            _ => {}
        }
    }
    fn call(&mut self, can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::OP_CALL, OpCode::Constant(arg_count));
    }
    fn dot(&mut self, can_assign: bool) {
        self.consume(
            TokenType::TOKEN_IDENTIFIER,
            "Expect property name after a '.'.".to_owned(),
        );
        let name = self.identifier_constant(self.previous.clone());
        if can_assign && self.match_token(TokenType::TOKEN_EQUAL) {
            self.expression();
            self.emit_bytes(OpCode::OP_SET_PROPERTY, OpCode::Constant(name));
        } else if self.match_token(TokenType::TOKEN_LEFT_PAREN) {
            let arg_count = self.argument_list();
            self.emit_bytes(OpCode::OP_INVOKE, OpCode::Constant(name));
            self.emit_byte(OpCode::Constant(arg_count));
        } else {
            self.emit_bytes(OpCode::OP_GET_PROPERTY, OpCode::Constant(name));
        }
    }
    fn and(&mut self, can_assign: bool) {
        let end_junp = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::OP_POP);
        self.parse_precedence(Precedence::PREC_AND);
        self.patch_jump(end_junp);
    }
    fn or(&mut self, can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::OP_POP);
        self.parse_precedence(Precedence::PREC_OR);
        self.patch_jump(end_jump);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
    }
    fn groping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(
            TokenType::TOKEN_LEFT_PAREN,
            "Expect ')' after expression.".to_owned(),
        );
    }
    fn unary(&mut self, can_assign: bool) {
        self.parse_precedence(Precedence::PREC_UNARY);
        match self.previous.token_type {
            TokenType::TOKEN_BANG => self.emit_byte(OpCode::OP_NOT),
            TokenType::TOKEN_MINUS => self.emit_byte(OpCode::OP_NEGATE),
            _ => {}
        }
    }
    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }
    fn resolve_local(&mut self, name: Token) -> isize {
        for i in (0..self.locals.len() - 1).rev() {
            let local = self.locals[i].clone();
            if name == local.name {
                if local.depth == -1 {
                    self.error("Can't read local variable in its own initializer.".to_owned());
                }
                return i as isize;
            }
        }
        return -1;
    }
    fn add_local(&mut self, name: Token) {
        self.locals.push(Local {
            name,
            depth: -1,
            is_captured: false,
        });
    }
    fn add_upvalue(&mut self, index: isize, is_local: bool) -> isize {
        let upvalue_count = self.function.upvalue_count;
        for i in 0..upvalue_count {
            let upvalue = &self.upvalues[i as usize];
            if upvalue.index == index && upvalue.is_local == is_local {
                return i;
            }
        }
        self.upvalues[upvalue_count as usize].is_local = is_local;
        self.upvalues[upvalue_count as usize].index = index;
        return self.function.upvalue_count;
    }
    fn resolve_upvalue(&mut self, name: Token) -> isize {
        match &mut self.enclosing {
            None => {
                return -1;
            }
            Some(enclosing) => {
                let local = enclosing.resolve_local(name.clone());
                if local != -1 {
                    enclosing.locals[local as usize].is_captured = true;
                    return self.add_upvalue(local, true);
                }
                let upvalue = self.resolve_upvalue(name.clone());
                if upvalue != -1 {
                    return self.add_upvalue(upvalue, false);
                }
            }
        }
        return -1;
    }
    fn make_constant(&mut self, value: String) -> isize {
        self.current_chunk().add_constant(value.clone());
        return value.len() as isize;
    }
    fn identifier_constant(&mut self, name: Token) -> isize {
        return self.make_constant(name.lexeme.to_owned());
    }
    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let get_op: OpCode;
        let set_op: OpCode;
        let mut arg = self.resolve_local(name.clone());
        if arg != 1 {
            get_op = OpCode::OP_GET_LOCAL;
            set_op = OpCode::OP_SET_LOCAL;
        } else {
            arg = self.resolve_upvalue(name.clone());
            if arg != -1 {
                get_op = OpCode::OP_GET_UPVALUE;
                set_op = OpCode::OP_SET_UPVALUE;
            } else {
                arg = self.identifier_constant(name.clone());
                get_op = OpCode::OP_GET_GLOBAL;
                set_op = OpCode::OP_SET_GLOBAL;
            }
        }
        if (can_assign && self.match_token(TokenType::TOKEN_EQUAL)) {
            self.expression();
            self.emit_bytes(set_op, OpCode::Constant(arg));
        } else {
            self.emit_bytes(get_op, OpCode::Constant(arg));
        }
    }
    fn synthetic_token(&self, text: String) -> Token {
        Token {
            token_type: TokenType::TOKEN_SYNTHETIC,
            lexeme: text.to_owned(),
            line: 0,
        }
    }
    fn argument_list(&mut self) -> isize {
        let mut arg_count: isize = 0;
        if !self.check(TokenType::TOKEN_RIGHT_PAREN) {
            loop {
                self.expression();
                arg_count += 1;
                if self.match_token(TokenType::TOKEN_COMMA) {
                    break;
                }
            }
        }
        return arg_count;
    }
    fn super_(&mut self, can_assign: bool) {
        match &self.class_compiler {
            None => {
                self.error("Can't use 'super' outside of a class.".to_owned());
            }
            Some(class_compiler) => {
                if !class_compiler.has_super_class {
                    self.error("Can't use 'super' in a class with no superclass.".to_owned());
                }
                self.consume(TokenType::TOKEN_DOT, "Expect '.' after 'super'.".to_owned());
                self.consume(
                    TokenType::TOKEN_IDENTIFIER,
                    "Expect superclass method name.".to_owned(),
                );
                let name = self.identifier_constant(self.previous.clone());

                self.named_variable(self.synthetic_token("this".to_owned()), false);
                if self.match_token(TokenType::TOKEN_LEFT_PAREN) {
                    let arg_count = self.argument_list();
                    self.named_variable(self.synthetic_token("super".to_owned()), false);
                    self.emit_bytes(OpCode::OP_SUPER_INVOKE, OpCode::Constant(name));
                    self.emit_byte(OpCode::Constant(arg_count));
                } else {
                    self.named_variable(self.synthetic_token("super".to_owned()), false);
                    self.emit_bytes(OpCode::OP_GET_SUPER, OpCode::Constant(name));
                }
            }
        }
    }
    fn this(&mut self, can_assign: bool) {
        match &self.class_compiler {
            None => {
                self.error("Can't use 'this' outside of a class.".to_owned());
            }
            Some(class_compiler) => {
                self.variable(false);
            }
        }
    }
    fn binary(&mut self, can_assign: bool) {
        let token_type = self.previous.token_type.clone();
        let parse_rule = get_rule(token_type.clone());
        self.parse_precedence(parse_rule.precedence.next());
        match token_type {
            TokenType::TOKEN_BANG_EQUAL => self.emit_bytes(OpCode::OP_EQUAL, OpCode::OP_NOT),
            TokenType::TOKEN_EQUAL_EQUAL => self.emit_byte(OpCode::OP_EQUAL),
            TokenType::TOKEN_GREATER => self.emit_byte(OpCode::OP_GREATER),
            TokenType::TOKEN_GREATER_EQUAL => self.emit_bytes(OpCode::OP_LESS, OpCode::OP_NOT),
            TokenType::TOKEN_LESS => self.emit_byte(OpCode::OP_LESS),
            TokenType::TOKEN_LESS_EQUAL => self.emit_bytes(OpCode::OP_GREATER, OpCode::OP_NOT),
            TokenType::TOKEN_PLUS => self.emit_byte(OpCode::OP_ADD),
            TokenType::TOKEN_MINUS => self.emit_byte(OpCode::OP_SUBTRACT),
            TokenType::TOKEN_STAR => self.emit_byte(OpCode::OP_MULTIPLY),
            TokenType::TOKEN_SLASH => self.emit_byte(OpCode::OP_DIVIDE),
            _ => {}
        }
    }
    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.current.token_type != TokenType::TOKEN_EOF {
            if self.previous.token_type == TokenType::TOKEN_SEMICOLON {
                return;
            }
            match self.current.token_type {
                TokenType::TOKEN_CLASS
                | TokenType::TOKEN_FUN
                | TokenType::TOKEN_VAR
                | TokenType::TOKEN_FOR
                | TokenType::TOKEN_IF
                | TokenType::TOKEN_WHILE
                | TokenType::TOKEN_PRINT
                | TokenType::TOKEN_RETURN => {
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }
    fn declare_variable(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let name = self.previous.clone();
        for local in self.locals.clone() {
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }
            if name == local.name {
                self.error("Already a variable with this name in this scope.".to_owned());
            }
        }
        self.add_local(name)
    }
    fn mark_initialized(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        let mut local = self.locals.pop().unwrap();
        local.depth = self.scope_depth;
        self.locals.push(local);
    }
    fn define_variable(&mut self, global: isize) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_byte(OpCode::DefineGlobal(global));
    }
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }
    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        for local in self.locals.clone() {
            if local.depth > self.scope_depth && local.is_captured {
                self.emit_byte(OpCode::OP_CLOSE_UPVALUE);
            }
            if local.depth > self.scope_depth && !local.is_captured {
                self.emit_byte(OpCode::OP_POP);
            }
        }
    }
    fn end_compiler(&mut self) -> ObjFunction {
        self.emit_return();
        let function = self.function.clone();
        return *function;
    }
    fn parse_variable(&mut self, error_masage: String) -> isize {
        self.consume(TokenType::TOKEN_IDENTIFIER, error_masage);
        self.declare_variable();
        if self.scope_depth > 0 {
            return 0;
        } else {
            return self.identifier_constant(self.previous.clone());
        }
    }
    fn expression_statement(&mut self) {
        self.expression();
        self.consume(
            TokenType::TOKEN_SEMICOLON,
            "Expect ';' after expression.".to_owned(),
        );
        self.emit_byte(OpCode::OP_POP);
    }
    fn return_statement(&mut self) {
        if self.function_type == FunctionType::TYPE_SCRIPT {
            self.error("Can't return from top-level code.".to_owned());
        }
        if self.match_token(TokenType::TOKEN_SEMICOLON) {
            self.emit_return();
        } else {
            if self.function_type == FunctionType::TYPE_INITIALIZER {
                self.error("Can't return a value from an initializer.".to_owned());
            }
            self.expression();
            self.consume(
                TokenType::TOKEN_SEMICOLON,
                "Expect ';' after return value.".to_owned(),
            );
            self.emit_byte(OpCode::OP_RETURN);
        }
    }
    fn print_statement(&mut self) {
        self.expression();
        self.consume(
            TokenType::TOKEN_SEMICOLON,
            "Expect ';' after value.".to_owned(),
        );
        self.emit_byte(OpCode::OP_PRINT);
    }
    fn if_statement(&mut self) {
        self.consume(
            TokenType::TOKEN_LEFT_PAREN,
            "Expect '(' after if.".to_owned(),
        );
        self.expression();
        self.consume(
            TokenType::TOKEN_RIGHT_PAREN,
            "Expect ')' after condition.".to_owned(),
        );
        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::OP_POP);
        self.statement();
        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::OP_POP);
        if self.match_token(TokenType::TOKEN_ELSE) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }
    fn emit_loop(&mut self, loop_start: i32) {
        self.emit_byte(OpCode::OP_LOOP);
        let offset = self.current_chunk().count - loop_start + 2;
        self.emit_byte(OpCode::Constant(((offset >> 8) & 0xFF) as isize));
        self.emit_byte(OpCode::Constant((offset & 0xFF) as isize));
    }
    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(
            TokenType::TOKEN_LEFT_PAREN,
            "Expect '(' after 'for'.".to_owned(),
        );
        if self.match_token(TokenType::TOKEN_SEMICOLON) {
        } else if self.match_token(TokenType::TOKEN_VAR) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }
        let mut loop_start = self.current_chunk().count;
        let exit_jump: isize = -1;
        if !self.match_token(TokenType::TOKEN_SEMICOLON) {
            self.expression();
            self.consume(TokenType::TOKEN_SEMICOLON, "Expect ';'.".to_owned());
            let exit_jump = self.emit_jump(OpCode::JumpIfFalse);
            self.emit_byte(OpCode::OP_POP);
        }
        if !self.match_token(TokenType::TOKEN_RIGHT_PAREN) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().count;
            self.expression();
            self.emit_byte(OpCode::OP_POP);
            self.consume(
                TokenType::TOKEN_RIGHT_PAREN,
                "Expect ')' after for clauses.".to_owned(),
            );
            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }
        self.statement();
        self.emit_loop(loop_start);
        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_byte(OpCode::OP_POP);
        }
        self.end_scope();
    }
    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().count;
        self.consume(
            TokenType::TOKEN_LEFT_PAREN,
            "Expect '(' after 'while'.".to_owned(),
        );
        self.expression();
        self.consume(
            TokenType::TOKEN_RIGHT_PAREN,
            "Expect ')' after condition.".to_owned(),
        );
        let exit_jomp = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::OP_POP);
        self.statement();
        self.emit_loop(loop_start);
        self.patch_jump(exit_jomp);
        self.emit_byte(OpCode::OP_POP);
    }
    fn statement(&mut self) {
        if self.match_token(TokenType::TOKEN_PRINT) {
            self.print_statement();
        } else if self.match_token(TokenType::TOKEN_IF) {
            self.if_statement();
        } else if self.match_token(TokenType::TOKEN_WHILE) {
            self.while_statement();
        } else if self.match_token(TokenType::TOKEN_FOR) {
            self.for_statement();
        } else if self.match_token(TokenType::TOKEN_LEFT_BRACE) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else if self.match_token(TokenType::TOKEN_RETURN) {
            self.return_statement();
        } else {
            self.expression_statement();
        }
    }
    fn function(&mut self, function_type: FunctionType) {
        let mut compiler = Compiler::new(Some(Box::new(self.clone())), function_type);
        compiler.begin_scope();
        compiler.consume(
            TokenType::TOKEN_LEFT_PAREN,
            "Expect '(' after function name.".to_owned(),
        );
        if compiler.check(TokenType::TOKEN_RIGHT_PAREN) {
            loop {
                compiler.function.arity += 1;
                let constat = compiler.parse_variable("Expect parameter name.".to_owned());
                compiler.define_variable(constat);
                if compiler.match_token(TokenType::TOKEN_COMMA) {
                    break;
                }
            }
        }
        compiler.consume(
            TokenType::TOKEN_RIGHT_PAREN,
            "Expect ')' after parameters.".to_owned(),
        );
        compiler.consume(
            TokenType::TOKEN_LEFT_BRACE,
            "Expect '{' before function body.".to_owned(),
        );
        compiler.block();
        let function: ObjFunction = compiler.end_compiler();
        let constant = self.make_constant(function.name);
        self.emit_bytes(OpCode::OP_CLOSURE, OpCode::Constant(constant));
        for i in 0..function.upvalue_count {
            let is_local_byte = if compiler.upvalues[i as usize].is_local {
                1
            } else {
                0
            };
            self.emit_byte(OpCode::Constant(is_local_byte));
            self.emit_byte(OpCode::Constant(compiler.upvalues[i as usize].index));
        }
    }
    fn method(&mut self) {
        self.consume(
            TokenType::TOKEN_IDENTIFIER,
            "Expect method name.".to_owned(),
        );
        let constant = self.identifier_constant(self.previous.clone());
        let mut type_ = FunctionType::TYPE_METHOD;
        if self.previous.lexeme == "init" {
            type_ = FunctionType::TYPE_INITIALIZER;
        }
        self.function(type_);
        self.emit_bytes(OpCode::OP_METHOD, OpCode::Constant(constant));
    }
    fn function_declaration(&mut self) {
        let global = self.parse_variable("Expect function name".to_owned());
        self.mark_initialized();
        self.function(FunctionType::TYPE_FUNCTION);
        self.define_variable(global);
    }
    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name".to_owned());
        if self.match_token(TokenType::TOKEN_EQUAL) {
            self.expression();
        } else {
            self.emit_byte(OpCode::OP_NIL);
        }
        self.consume(
            TokenType::TOKEN_SEMICOLON,
            "Expect ';' after expression.".to_owned(),
        );
        self.define_variable(global);
    }
    fn class_declaration(&mut self) {
        self.consume(TokenType::TOKEN_IDENTIFIER, "Expect class name.".to_owned());
        let class_name = self.previous.clone();
        let name_constant = self.identifier_constant(self.previous.clone());
        self.declare_variable();
        self.emit_bytes(OpCode::OP_CLASS, OpCode::Constant(name_constant));
        self.define_variable(name_constant);
        let class_compiler = ClassCompiler {
            enclosing: self.class_compiler.clone(),
            has_super_class: false,
        };
        self.class_compiler = Some(Box::new(class_compiler.clone()));
        if self.match_token(TokenType::TOKEN_LESS) {
            self.consume(
                TokenType::TOKEN_IDENTIFIER,
                "Expected superclass name.".to_owned(),
            );
            self.variable(false);
            if class_name == self.previous {
                self.error("A class can't ingerit from itself".to_owned());
            }
            self.begin_scope();
            self.define_variable(0);
            self.named_variable(class_name.clone(), false);
            self.emit_byte(OpCode::OP_INHERIT);
            let mut class_compiler_tmp = self.class_compiler.clone().unwrap();
            class_compiler_tmp.has_super_class = true;
            self.class_compiler = Some(class_compiler_tmp.clone());
        }
        self.named_variable(class_name.clone(), false);
        self.consume(
            TokenType::TOKEN_LEFT_BRACE,
            "Expect '{' before class body.".to_owned(),
        );
        while self.check(TokenType::TOKEN_RIGHT_BRACE) && !self.check(TokenType::TOKEN_EOF) {
            self.method();
        }
        self.consume(
            TokenType::TOKEN_RIGHT_BRACE,
            "Expect '}' after class body.".to_owned(),
        );
        self.emit_byte(OpCode::OP_POP);
        if self.class_compiler.clone().unwrap().has_super_class {
            self.end_scope();
        }
        self.class_compiler = self.class_compiler.clone().unwrap().enclosing;
    }
    fn declaration(&mut self) {
        if self.match_token(TokenType::TOKEN_CLASS) {
            self.class_declaration();
        } else if self.match_token(TokenType::TOKEN_FUN) {
            self.function_declaration();
        } else if self.match_token(TokenType::TOKEN_VAR) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize()
        }
    }
    fn block(&mut self) {
        while self.check(TokenType::TOKEN_RIGHT_BRACE) && !self.check(TokenType::TOKEN_EOF) {
            self.declaration()
        }
        self.consume(
            TokenType::TOKEN_RIGHT_BRACE,
            "Expect '}' after block.".to_owned(),
        );
    }
    fn compile(&mut self, source: String) -> ObjFunction {
        let scaner = Scanner::new(source);
        self.scanner = Some(scaner);
        self.advance();
        while !self.match_token(TokenType::TOKEN_EOF) {
            self.declaration();
        }
        let function = self.end_compiler();
        return function;
    }
}
// TODO add unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expresion1() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::Constant(1),
                OpCode::OP_ADD,
                OpCode::OP_POP,
                OpCode::OP_NIL,
                OpCode::OP_RETURN,
            ],
            lines: vec![1, 1, 1, 1, 1, 1],
            constants: vec!["2".into(), "2".into()],
            count: 6,
        };
        let source = "2 + 2;".to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TYPE_SCRIPT);
        compiler.compile(source);
        let chunk = compiler.current_chunk();
        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_expresion2() {
        // TODO fix it, somthink with definif variables
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::Constant(1),
                OpCode::OP_ADD,
                OpCode::DefineGlobal(2),
                OpCode::OP_NIL,
                OpCode::OP_RETURN,
            ],
            lines: vec![1, 1, 1, 1, 1, 1],
            constants: vec!["2".into(), "2".into(), "x".into()],
            count: 6,
        };
        let source = "var x = 2 + 2;".to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TYPE_SCRIPT);
        compiler.compile(source);
        let chunk = compiler.current_chunk();
        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_statement() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::OP_PRINT,
                OpCode::OP_NIL,
                OpCode::OP_RETURN,
            ],
            lines: vec![1, 1, 1, 1],
            constants: vec!["test".into()],
            count: 4,
        };
        let source = "print \"test\";".to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TYPE_SCRIPT);
        compiler.compile(source);
        let chunk = compiler.current_chunk();
        assert_eq!(chunk, &expected_chunk);
    }

    #[test]
    fn test_function() {
        //TODO fix it
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::OP_PRINT,
                OpCode::OP_NIL,
                OpCode::OP_RETURN,
            ],
            lines: vec![1, 1, 1, 1],
            constants: vec!["test".into()],
            count: 4,
        };
        let source = "fun test(x) {
                                var y = 2 + x;
                                return y;
                            }
                            print test(10);
        "
        .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TYPE_SCRIPT);
        compiler.compile(source);
        let chunk = compiler.current_chunk();
        assert_eq!(chunk, &expected_chunk);
    }
}
