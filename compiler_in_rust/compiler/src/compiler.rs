use std::collections::HashMap;

use crate::{
    chunks::{Chunk, OpCode},
    object::ObjFunction,
    scaner::{Scanner, Token, TokenType},
};

// struct Parser {
//     scanner: Scanner,
//     current: Token,
//     previous: Token,
//     had_error: bool,
//     panic_mode: bool,
// }
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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

struct Upvalue {
    index: isize,
    is_local: bool,
}

#[derive(PartialEq)]
enum FunctionType {
    TYPE_FUNCTION,
    TYPE_SCRIPT,
    TYPE_METHOD,
    TYPE_INITIALIZER,
}

struct Compiler<'a> {
    enclosing: Option<Box<Compiler<'a>>>,
    class_compiler: Option<Box<ClassCompiler>>,
    function: Box<ObjFunction<'a>>,
    function_type: FunctionType,
    locals: Vec<Local>,
    upvalues: Vec<Upvalue>,
    scope_depth: usize,
    scanner: Scanner,
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

impl<'a> Compiler<'a> {
    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.line;
        self.current_chunk().write_chunk(byte, line);
    }
    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }
    fn emit_jump(&mut self, instruction: u8) -> i32 {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        return self.current_chunk().count - 2;
    }
    fn patch_junp(&mut self, offset: i32) {
        let jump = self.current_chunk().count - offset - 2;
        self.current_chunk().code[offset as usize] = ((jump >> 8) & 0xFF) as u8;
        self.current_chunk().code[(offset + 1) as usize] = (jump & 0xFF) as u8;
    }
    fn emit_constant(&mut self, value: String) {
        let constant_value = self.make_constant(value) as u8;
        self.emit_bytes(OpCode::OP_CONSTANT as u8, constant_value);
    }
    fn emit_return(&mut self) {
        if self.function_type == FunctionType::TYPE_INITIALIZER {
            self.emit_bytes(OpCode::OP_GET_LOCAL as u8, 0);
        } else {
            self.emit_byte(OpCode::OP_NIL as u8);
        }
        self.emit_byte(OpCode::OP_RETURN as u8);
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
            self.current = self.scanner.scan_token();
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
        self.emit_constant(self.previous.lexeme.clone());
    }
    fn number(&mut self, can_assign: bool) {
        self.emit_constant(self.previous.lexeme.clone());
    }
    fn literal(&mut self, can_assign: bool) {
        match self.previous.token_type {
            TokenType::TOKEN_FALSE => self.emit_byte(OpCode::OP_FALSE as u8),
            TokenType::TOKEN_TRUE => self.emit_byte(OpCode::OP_TRUE as u8),
            TokenType::TOKEN_NIL => self.emit_byte(OpCode::OP_NIL as u8),
            _ => {}
        }
    }
    fn call(&mut self, can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::OP_CALL as u8, arg_count);
    }
    fn dot(&mut self, can_assign: bool) {
        self.consume(
            TokenType::TOKEN_IDENTIFIER,
            "Expect property name after a '.'.".to_owned(),
        );
        let name = self.identifier_constant(self.previous.clone());
        if can_assign && self.match_token(TokenType::TOKEN_EQUAL) {
            self.expression();
            self.emit_bytes(OpCode::OP_SET_PROPERTY as u8, name as u8);
        } else if self.match_token(TokenType::TOKEN_LEFT_PAREN) {
            let arg_count = self.argument_list();
            self.emit_bytes(OpCode::OP_INVOKE as u8, name as u8);
            self.emit_byte(arg_count);
        } else {
            self.emit_bytes(OpCode::OP_GET_PROPERTY as u8, name as u8);
        }
    }
    fn and(&mut self, can_assign: bool) {
        let end_junp = self.emit_jump(OpCode::OP_JUMP_IF_FALSE as u8);
        self.emit_byte(OpCode::OP_POP as u8);
        self.parse_precedence(Precedence::PREC_AND);
        self.patch_junp(end_junp);
    }
    fn or(&mut self, can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::OP_JUMP_IF_FALSE as u8);
        let end_jump = self.emit_jump(OpCode::OP_JUMP as u8);
        self.patch_junp(else_jump);
        self.emit_byte(OpCode::OP_POP as u8);
        self.parse_precedence(Precedence::PREC_OR);
        self.patch_junp(end_jump);
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
            TokenType::TOKEN_BANG => self.emit_byte(OpCode::OP_NOT as u8),
            TokenType::TOKEN_MINUS => self.emit_byte(OpCode::OP_NEGATE as u8),
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
            self.emit_bytes(set_op as u8, arg as u8);
        } else {
            self.emit_bytes(get_op as u8, arg as u8);
        }
    }
    fn synthetic_token(&self, text: String) -> Token {
        Token {
            token_type: TokenType::TOKEN_SYNTHETIC,
            lexeme: text.to_owned(),
            line: 0,
        }
    }
    fn argument_list(&mut self) -> u8 {
        let mut arg_count: u8 = 0;
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
                    self.emit_bytes(OpCode::OP_SUPER_INVOKE as u8, name as u8);
                    self.emit_byte(arg_count);
                } else {
                    self.named_variable(self.synthetic_token("super".to_owned()), false);
                    self.emit_bytes(OpCode::OP_GET_SUPER as u8, name as u8);
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
            TokenType::TOKEN_BANG_EQUAL => {
                self.emit_bytes(OpCode::OP_EQUAL as u8, OpCode::OP_NOT as u8)
            }
            TokenType::TOKEN_EQUAL_EQUAL => self.emit_byte(OpCode::OP_EQUAL as u8),
            TokenType::TOKEN_GREATER => self.emit_byte(OpCode::OP_GREATER as u8),
            TokenType::TOKEN_GREATER_EQUAL => {
                self.emit_bytes(OpCode::OP_LESS as u8, OpCode::OP_NOT as u8)
            }
            TokenType::TOKEN_LESS => self.emit_byte(OpCode::OP_LESS as u8),
            TokenType::TOKEN_LESS_EQUAL => {
                self.emit_bytes(OpCode::OP_GREATER as u8, OpCode::OP_NOT as u8)
            }
            TokenType::TOKEN_PLUS => self.emit_byte(OpCode::OP_ADD as u8),
            TokenType::TOKEN_MINUS => self.emit_byte(OpCode::OP_SUBTRACT as u8),
            TokenType::TOKEN_STAR => self.emit_byte(OpCode::OP_MULTIPLY as u8),
            TokenType::TOKEN_SLASH => self.emit_byte(OpCode::OP_DIVIDE as u8),
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
    fn class_declaration(&mut self) {
        self.consume(TokenType::TOKEN_IDENTIFIER, "Expect class name.".to_owned());
        let class_name = self.previous.clone();
        let name_constant = self.identifier_constant(self.previous.clone());
        //self.declare_variable() TODO
        self.emit_bytes(OpCode::OP_CLASS as u8, name_constant as u8);
        //self.define_variable(name_constant) TODO
        let mut class_compiler = ClassCompiler {
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
            // self.begin_scope() TODO
            // self.define_variable(0) TODO
            self.named_variable(class_name.clone(), false);
            self.emit_byte(OpCode::OP_INHERIT as u8);
            // self.class_compiler.unwrap().has_super_class = true;  TODO
        }
        self.named_variable(class_name.clone(), false);
        self.consume(
            TokenType::TOKEN_LEFT_BRACE,
            "Expect '{' before class body.".to_owned(),
        );
        while self.check(TokenType::TOKEN_RIGHT_BRACE) && !self.check(TokenType::TOKEN_EOF) {
            //self.method(); TODO
        }
        self.consume(
            TokenType::TOKEN_RIGHT_BRACE,
            "Expect '}' after class body.".to_owned(),
        );
        self.emit_byte(OpCode::OP_POP as u8);
        if self.class_compiler.clone().unwrap().has_super_class {
            //self.end_scope(); TODO
        }
        self.class_compiler = self.class_compiler.clone().unwrap().enclosing;
    }
    fn declaration(&mut self) {
        if self.match_token(TokenType::TOKEN_CLASS) {
            //self.class_declaration(); TODO
        } else if self.match_token(TokenType::TOKEN_FUN) {
            //self.function_declaration(); TODO
        } else if self.match_token(TokenType::TOKEN_VAR) {
            // self.var_declaration(); TODO
        } else {
            // self.statement() TODO
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
}
