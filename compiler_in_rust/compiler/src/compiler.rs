use std::collections::HashMap;

use crate::chunks::Value;
use crate::{
    chunks::{Chunk, OpCode},
    object::ObjFunction,
    scaner::{Scanner, Token, TokenType},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Precedence {
    PrecNone,
    PrecAssignment,
    PrecOr,
    PrecAnd,
    PrecEquality,
    PrecComparison,
    PrecTerm,
    PrecFactor,
    PrecUnary,
    PrecCall,
    PrecPrimary,
}

#[derive(Copy, Clone, Debug)]
enum ExprssionType {
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
    pub prefix: Option<ExprssionType>,
    pub infix: Option<ExprssionType>,
    pub precedence: Precedence,
}

lazy_static::lazy_static! {
    static ref RULES: HashMap<TokenType, ParseRule> = {
        let mut m = HashMap::new();

        m.insert(TokenType::TokenLeftParen,     ParseRule { prefix: Some(ExprssionType::GROUPING), infix: Some(ExprssionType::CALL),    precedence: Precedence::PrecCall });
        m.insert(TokenType::TokenRightParen,    ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenLeftBrace,     ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenRightBrace,    ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenDot,            ParseRule { prefix: None,                         infix: Some(ExprssionType::DOT),     precedence: Precedence::PrecCall });
        m.insert(TokenType::TokenMinus,          ParseRule { prefix: Some(ExprssionType::UNARY),    infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecTerm });
        m.insert(TokenType::TokenPlus,           ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecTerm });
        m.insert(TokenType::TokenSemicolon,      ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenSlash,          ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecFactor });
        m.insert(TokenType::TokenStar,           ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecFactor });
        m.insert(TokenType::TokenBang,           ParseRule { prefix: Some(ExprssionType::UNARY),    infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenBangEqual,     ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecEquality });
        m.insert(TokenType::TokenEqual,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenEqualEqual,    ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecEquality });
        m.insert(TokenType::TokenGreater,        ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecComparison });
        m.insert(TokenType::TokenGreaterEqual,  ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecComparison });
        m.insert(TokenType::TokenLess,           ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecComparison });
        m.insert(TokenType::TokenLessEqual,     ParseRule { prefix: None,                         infix: Some(ExprssionType::BINARY),  precedence: Precedence::PrecComparison });
        m.insert(TokenType::TokenIdentifier,     ParseRule { prefix: Some(ExprssionType::VARIABLE), infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenString,         ParseRule { prefix: Some(ExprssionType::STRING),   infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenNumber,         ParseRule { prefix: Some(ExprssionType::NUMBER),   infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenAnd,            ParseRule { prefix: None,                         infix: Some(ExprssionType::AND),     precedence: Precedence::PrecAnd });
        m.insert(TokenType::TokenClass,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenElse,           ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenFalse,          ParseRule { prefix: Some(ExprssionType::LITERAL),  infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenFor,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenFun,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenIf,             ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenNil,            ParseRule { prefix: Some(ExprssionType::LITERAL),  infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenOr,             ParseRule { prefix: None,                         infix: Some(ExprssionType::OR),      precedence: Precedence::PrecOr });
        m.insert(TokenType::TokenPrint,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenReturn,         ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenSuper,          ParseRule { prefix: Some(ExprssionType::SUPER),    infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenThis,           ParseRule { prefix: Some(ExprssionType::THIS),     infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenTrue,           ParseRule { prefix: Some(ExprssionType::LITERAL),  infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenVar,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenWhile,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenError,          ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });
        m.insert(TokenType::TokenEof,            ParseRule { prefix: None,                         infix: None,                        precedence: Precedence::PrecNone });

        m
    };
}
fn get_rule(token_type: TokenType) -> ParseRule {
    RULES.get(&token_type).copied().unwrap_or(ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::PrecNone,
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

#[derive(PartialEq, Clone, Copy)]
enum FunctionType {
    TypeFunction,
    TypeScript,
    TypeMethod,
    TypeInitializer,
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
            Precedence::PrecNone => Precedence::PrecAssignment,
            Precedence::PrecAssignment => Precedence::PrecOr,
            Precedence::PrecOr => Precedence::PrecAnd,
            Precedence::PrecAnd => Precedence::PrecEquality,
            Precedence::PrecEquality => Precedence::PrecComparison,
            Precedence::PrecComparison => Precedence::PrecTerm,
            Precedence::PrecTerm => Precedence::PrecFactor,
            Precedence::PrecFactor => Precedence::PrecUnary,
            Precedence::PrecUnary => Precedence::PrecCall,
            Precedence::PrecCall => Precedence::PrecPrimary,
            Precedence::PrecPrimary => Precedence::PrecPrimary, // highest
        }
    }
}

impl Compiler {
    fn new(enclosing: Option<Box<Compiler>>, function_type: FunctionType) -> Self {
        let mut function = ObjFunction::new();
        if function_type != FunctionType::TypeScript {
            if let Some(ref parent) = enclosing {
                function.name = parent.previous.lexeme.clone();
            }
        }

        let mut compiler = Compiler {
            enclosing,
            function_type,
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
            function: Box::new(function),
            scanner: None,
            current: Token {
                token_type: TokenType::TokenSynthetic,
                lexeme: "".to_owned(),
                line: 0,
            },
            previous: Token {
                token_type: TokenType::TokenSynthetic,
                lexeme: "".to_owned(),
                line: 0,
            },
            had_error: false,
            panic_mode: false,
            class_compiler: None,
        };

        let local_name = match function_type {
            FunctionType::TypeMethod | FunctionType::TypeInitializer => Token {
                token_type: TokenType::TokenThis,
                lexeme: "this".to_owned(),
                line: 0,
            },
            _ => Token {
                token_type: TokenType::TokenSynthetic,
                lexeme: "".to_owned(),
                line: 0,
            },
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
    fn emit_jump(&mut self, instruction: fn(i16) -> OpCode) -> isize {
        self.emit_byte(instruction(0));
        (self.current_chunk().count - 1) as isize
    }
    fn patch_jump(&mut self, jump_index: isize) {
        let offset = self.current_chunk().count - jump_index - 1;

        let opcode = &mut self.current_chunk().code[jump_index as usize];

        match opcode {
            OpCode::Jump(o) | OpCode::JumpIfFalse(o) | OpCode::Loop(o) => {
                *o = offset as i16;
            }
            _ => panic!("Invalid jump patch"),
        }
    }
    fn emit_constant(&mut self, value: Value) {
        let constant_index = self.current_chunk().add_constant(value);
        self.emit_byte(OpCode::Constant(constant_index));
    }
    fn emit_return(&mut self) {
        if self.function_type == FunctionType::TypeInitializer {
            self.emit_byte(OpCode::GetLocal(0));
        } else {
            self.emit_byte(OpCode::OpNil);
        }
        self.emit_byte(OpCode::OpReturn);
    }
    fn error_et(&mut self, token: Token, message: String) {
        if self.panic_mode {
            return;
        }
        eprint!("[line {}]", token.line);
        match token.token_type {
            TokenType::TokenEof => {
                eprint!(" at end");
            }
            TokenType::TokenError => {}
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
            if self.current.token_type != TokenType::TokenError {
                break;
            } else {
                self.error_at_current(self.current.lexeme.clone());
            }
        }
    }
    fn consume(&mut self, token_type: TokenType, message: String) {
        println!("{:?} == {:?}", self.current.token_type, token_type);
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
            false
        } else {
            self.advance();
            true
        }
    }
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        let mut can_assign = precedence <= Precedence::PrecAssignment;
        let prefix_rule = match get_rule(self.previous.token_type.clone()).prefix {
            Some(rule) => rule,
            _ => {
                return;
            }
        };
        match prefix_rule {
            ExprssionType::GROUPING => self.grouping(can_assign),
            ExprssionType::UNARY => self.unary(can_assign),
            ExprssionType::VARIABLE => self.variable(can_assign),
            ExprssionType::STRING => self.string(can_assign),
            ExprssionType::NUMBER => self.number(can_assign),
            ExprssionType::LITERAL => self.literal(can_assign),
            ExprssionType::SUPER => self.super_(can_assign),
            ExprssionType::THIS => self.this(can_assign),
            _ => self.error("Incorrect prefix rule".to_owned()),
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
                ExprssionType::CALL => self.call(false),
                ExprssionType::DOT => self.dot(can_assign),
                ExprssionType::AND => self.and(false),
                ExprssionType::OR => self.or(false),
                ExprssionType::BINARY => self.binary(false),
                _ => self.error("Incorrect infix rule".to_owned()),
            }
            if can_assign && self.match_token(TokenType::TokenEqual) {
                self.error("Invalid assigment target.".to_owned());
            }
        }
    }
    fn string(&mut self, can_assign: bool) {
        let value = self.previous.lexeme.clone().replace("\"", "");
        self.emit_constant(Value::String(value));
    }
    fn number(&mut self, can_assign: bool) {
        let value: f64 = self.previous.lexeme.parse().unwrap_or(0.0);
        self.emit_constant(Value::Number(value));
    }
    fn literal(&mut self, can_assign: bool) {
        match self.previous.token_type {
            TokenType::TokenFalse => self.emit_byte(OpCode::OpFalse),
            TokenType::TokenTrue => self.emit_byte(OpCode::OpTrue),
            TokenType::TokenNil => self.emit_byte(OpCode::OpNil),
            _ => {}
        }
    }
    fn call(&mut self, _can_assign: bool) {
        let arg_count = self.argument_list();
        self.emit_byte(OpCode::Call(arg_count));
    }
    fn dot(&mut self, can_assign: bool) {
        self.consume(
            TokenType::TokenIdentifier,
            "Expect property name after a '.'.".to_owned(),
        );
        let name = self.identifier_constant_once(&self.previous.clone());
        if can_assign && self.match_token(TokenType::TokenEqual) {
            self.expression();
            self.emit_byte(OpCode::SetProperty(name));
        } else if self.match_token(TokenType::TokenLeftParen) {
            let arg_count = self.argument_list();
            self.emit_byte(OpCode::Invoke(name, arg_count));
        } else {
            self.emit_byte(OpCode::GetProperty(name));
        }
    }
    fn and(&mut self, can_assign: bool) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::OpPop);
        self.parse_precedence(Precedence::PrecAnd);
        self.patch_jump(end_jump);
    }
    fn or(&mut self, can_assign: bool) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse);
        let end_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::OpPop);
        self.parse_precedence(Precedence::PrecOr);
        self.patch_jump(end_jump);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PrecAssignment);
    }
    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(
            TokenType::TokenRightParen,
            "Expect ')' after expression.".to_owned(),
        );
    }
    fn unary(&mut self, can_assign: bool) {
        self.parse_precedence(Precedence::PrecUnary);
        match self.previous.token_type {
            TokenType::TokenBang => self.emit_byte(OpCode::OpNot),
            TokenType::TokenMinus => self.emit_byte(OpCode::OpNegate),
            _ => {}
        }
    }
    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.previous.clone(), can_assign);
    }
    fn resolve_local(&mut self, name: Token) -> isize {
        for i in (0..self.locals.len()).rev() {
            let local = &self.locals[i];
            if name.lexeme == local.name.lexeme {
                if local.depth == -1 {
                    self.error("Can't read local variable in its own initializer.".to_owned());
                }
                return i as isize;
            }
        }
        -1
    }
    fn add_local(&mut self, name: Token) {
        self.locals.push(Local {
            name,
            depth: -1,
            is_captured: false,
        });
    }
    fn add_upvalue(&mut self, index: isize, is_local: bool) -> isize {
        for (i, upvalue) in self.upvalues.iter().enumerate() {
            if upvalue.index == index && upvalue.is_local == is_local {
                return i as isize;
            }
        }

        self.upvalues.push(Upvalue { index, is_local });
        self.function.upvalue_count = self.upvalues.len() as isize;
        (self.upvalues.len() - 1) as isize
    }
    fn resolve_upvalue(&mut self, name: Token) -> isize {
        if let Some(enclosing) = &mut self.enclosing {
            let local = enclosing.resolve_local(name.clone());
            if local != -1 {
                enclosing.locals[local as usize].is_captured = true;
                return self.add_upvalue(local, true);
            }
            let upvalue = enclosing.resolve_upvalue(name.clone());
            if upvalue != -1 {
                return self.add_upvalue(upvalue, false);
            }
        }
        -1
    }
    fn make_constant(&mut self, value: Value) -> isize {
        self.current_chunk().add_constant(value);
        (self.current_chunk().constants.len() - 1) as isize
    }
    fn identifier_constant_once(&mut self, name: &Token) -> isize {
        let val = Value::String(name.lexeme.clone());
        if let Some(i) = self
            .current_chunk()
            .constants
            .iter()
            .position(|c| c == &val)
        {
            i as isize
        } else {
            self.make_constant(val)
        }
    }
    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let (get_op, set_op, arg) = {
            let local = self.resolve_local(name.clone());
            if local != -1 {
                (OpCode::GetLocal(local), OpCode::SetLocal(local), local)
            } else {
                let upvalue = self.resolve_upvalue(name.clone());
                if upvalue != -1 {
                    (
                        OpCode::GetUpvalue(upvalue),
                        OpCode::SetUpvalue(upvalue),
                        upvalue,
                    )
                } else {
                    let global = self.identifier_constant_once(&name);
                    (OpCode::GetGlobal(global), OpCode::SetGlobal(global), global)
                }
            }
        };

        if can_assign && self.match_token(TokenType::TokenEqual) {
            self.expression();
            self.emit_byte(set_op);
        } else {
            self.emit_byte(get_op);
        }
    }
    fn synthetic_token(&self, text: String) -> Token {
        Token {
            token_type: TokenType::TokenSynthetic,
            lexeme: text.to_owned(),
            line: 0,
        }
    }
    fn argument_list(&mut self) -> isize {
        let mut arg_count: isize = 0;

        if !self.check(TokenType::TokenRightParen) {
            loop {
                self.expression();
                arg_count += 1;

                if !self.match_token(TokenType::TokenComma) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::TokenRightParen,
            "Expect ')' after arguments.".to_owned(),
        );

        arg_count
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
                self.consume(TokenType::TokenDot, "Expect '.' after 'super'.".to_owned());
                self.consume(
                    TokenType::TokenIdentifier,
                    "Expect superclass method name.".to_owned(),
                );
                let name = self.identifier_constant_once(&self.previous.clone());

                self.named_variable(self.synthetic_token("this".to_owned()), false);
                if self.match_token(TokenType::TokenLeftParen) {
                    let arg_count = self.argument_list();
                    self.named_variable(self.synthetic_token("super".to_owned()), false);
                    self.emit_byte(OpCode::SuperInvoke(name, arg_count));
                } else {
                    self.named_variable(self.synthetic_token("super".to_owned()), false);
                    self.emit_byte(OpCode::GetSuper(name));
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
            TokenType::TokenBangEqual => self.emit_bytes(OpCode::OpEqual, OpCode::OpNot),
            TokenType::TokenEqualEqual => self.emit_byte(OpCode::OpEqual),
            TokenType::TokenGreater => self.emit_byte(OpCode::OpGreater),
            TokenType::TokenGreaterEqual => self.emit_bytes(OpCode::OpLess, OpCode::OpNot),
            TokenType::TokenLess => self.emit_byte(OpCode::OpLess),
            TokenType::TokenLessEqual => self.emit_bytes(OpCode::OpGreater, OpCode::OpNot),
            TokenType::TokenPlus => self.emit_byte(OpCode::OpAdd),
            TokenType::TokenMinus => self.emit_byte(OpCode::OpSubtract),
            TokenType::TokenStar => self.emit_byte(OpCode::OpMultiply),
            TokenType::TokenSlash => self.emit_byte(OpCode::OpDivide),
            _ => {}
        }
    }
    fn synchronize(&mut self) {
        self.panic_mode = false;
        while self.current.token_type != TokenType::TokenEof {
            if self.previous.token_type == TokenType::TokenSemicolon {
                return;
            }
            match self.current.token_type {
                TokenType::TokenClass
                | TokenType::TokenFun
                | TokenType::TokenVar
                | TokenType::TokenFor
                | TokenType::TokenIf
                | TokenType::TokenWhile
                | TokenType::TokenPrint
                | TokenType::TokenReturn => {
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
        for local in self.locals.clone().iter().rev() {
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
        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope_depth {
                break;
            }

            if local.is_captured {
                self.emit_byte(OpCode::OpCloseUpvalue);
            } else {
                self.emit_byte(OpCode::OpPop);
            }

            self.locals.pop();
        }
    }
    fn end_compiler(&mut self) -> ObjFunction {
        let ends_with_return = matches!(self.current_chunk().code.last(), Some(OpCode::OpReturn));
        if !ends_with_return {
            self.emit_return();
        }
        let function = self.function.clone();
        return *function;
    }
    fn parse_variable(&mut self, error_message: String) -> isize {
        self.consume(TokenType::TokenIdentifier, error_message);
        self.declare_variable();

        if self.scope_depth > 0 {
            -1
        } else {
            self.identifier_constant_once(&self.previous.clone())
        }
    }
    fn expression_statement(&mut self) {
        self.expression();
        self.consume(
            TokenType::TokenSemicolon,
            "Expect ';' after expression.".to_owned(),
        );
        self.emit_byte(OpCode::OpPop);
    }
    fn return_statement(&mut self) {
        if self.function_type == FunctionType::TypeScript {
            self.error("Can't return from top-level code.".to_owned());
        }
        if self.match_token(TokenType::TokenSemicolon) {
            self.emit_return();
        } else {
            if self.function_type == FunctionType::TypeInitializer {
                self.error("Can't return a value from an initializer.".to_owned());
            }
            self.expression();
            self.consume(
                TokenType::TokenSemicolon,
                "Expect ';' after return value.".to_owned(),
            );
            self.emit_byte(OpCode::OpReturn);
        }
    }
    fn print_statement(&mut self) {
        self.expression();
        self.consume(
            TokenType::TokenSemicolon,
            "Expect ';' after value.".to_owned(),
        );
        self.emit_byte(OpCode::OpPrint);
    }
    fn if_statement(&mut self) {
        self.consume(TokenType::TokenLeftParen, "Expect '(' after if.".to_owned());
        self.expression();
        self.consume(
            TokenType::TokenRightParen,
            "Expect ')' after condition.".to_owned(),
        );
        let then_jump = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::OpPop);
        self.statement();
        let else_jump = self.emit_jump(OpCode::Jump);
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::OpPop);
        if self.match_token(TokenType::TokenElse) {
            self.statement();
        }
        self.patch_jump(else_jump);
    }
    fn emit_loop(&mut self, loop_start: usize) {
        let offset = self.current_chunk().code.len() - loop_start + 1;
        self.emit_byte(OpCode::Loop(offset as i16));
    }
    fn for_statement(&mut self) {
        self.begin_scope();
        self.consume(
            TokenType::TokenLeftParen,
            "Expect '(' after 'for'.".to_owned(),
        );
        if self.match_token(TokenType::TokenSemicolon) {
        } else if self.match_token(TokenType::TokenVar) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }
        let mut loop_start = self.current_chunk().count;
        let mut exit_jump: isize = -1;
        if !self.match_token(TokenType::TokenSemicolon) {
            self.expression();
            self.consume(TokenType::TokenSemicolon, "Expect ';'.".to_owned());
            exit_jump = self.emit_jump(OpCode::JumpIfFalse) as isize;
            self.emit_byte(OpCode::OpPop);
        }
        if !self.match_token(TokenType::TokenRightParen) {
            let body_jump = self.emit_jump(OpCode::Jump);
            let increment_start = self.current_chunk().count;
            self.expression();
            self.emit_byte(OpCode::OpPop);
            self.consume(
                TokenType::TokenRightParen,
                "Expect ')' after for clauses.".to_owned(),
            );
            self.emit_loop(loop_start as usize);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }
        self.statement();
        self.emit_loop(loop_start as usize);
        if exit_jump != -1 {
            self.patch_jump(exit_jump);
            self.emit_byte(OpCode::OpPop);
        }
        self.end_scope();
    }
    fn while_statement(&mut self) {
        let loop_start = self.current_chunk().count;
        self.consume(
            TokenType::TokenLeftParen,
            "Expect '(' after 'while'.".to_owned(),
        );
        self.expression();
        self.consume(
            TokenType::TokenRightParen,
            "Expect ')' after condition.".to_owned(),
        );
        let exit_jomp = self.emit_jump(OpCode::JumpIfFalse);
        self.emit_byte(OpCode::OpPop);
        self.statement();
        self.emit_loop(loop_start as usize);
        self.patch_jump(exit_jomp);
        self.emit_byte(OpCode::OpPop);
    }
    fn statement(&mut self) {
        if self.match_token(TokenType::TokenPrint) {
            self.print_statement();
        } else if self.match_token(TokenType::TokenIf) {
            self.if_statement();
        } else if self.match_token(TokenType::TokenWhile) {
            self.while_statement();
        } else if self.match_token(TokenType::TokenFor) {
            self.for_statement();
        } else if self.match_token(TokenType::TokenLeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else if self.match_token(TokenType::TokenReturn) {
            self.return_statement();
        } else {
            self.expression_statement();
        }
    }
    fn function(&mut self, function_type: FunctionType) {
        let function_name = self.previous.lexeme.clone();
        let scanner = self.scanner.clone();
        let mut compiler = Compiler::new(Some(Box::new(self.clone())), function_type);
        compiler.scanner = scanner;
        compiler.current = self.current.clone();
        compiler.previous = self.previous.clone();

        compiler.begin_scope();
        compiler.consume(
            TokenType::TokenLeftParen,
            "Expect '(' after function name.".to_owned(),
        );

        if !compiler.check(TokenType::TokenRightParen) {
            loop {
                compiler.function.arity += 1;
                let _param_constant = compiler.parse_variable("Expect parameter name.".to_owned());
                compiler.define_variable(0); // This marks it initialized in the local scope
                if !compiler.match_token(TokenType::TokenComma) {
                    break;
                }
            }
        }
        compiler.consume(
            TokenType::TokenRightParen,
            "Expect ')' after parameters.".to_owned(),
        );
        compiler.consume(
            TokenType::TokenLeftBrace,
            "Expect '{' before function body.".to_owned(),
        );
        compiler.block();
        let mut function_obj = compiler.end_compiler();
        function_obj.name = function_name.clone();

        // Synchronize scanner state back to self
        self.scanner = compiler.scanner.clone();
        self.current = compiler.current.clone();
        self.previous = compiler.previous.clone();

        let function_constant = self
            .current_chunk()
            .add_constant(Value::Function(Box::new(function_obj)));
        self.emit_byte(OpCode::Closure(function_constant));

        for upvalue in &compiler.upvalues {
            let is_local_byte = if upvalue.is_local { 1 } else { 0 };
            self.emit_byte(OpCode::Constant(is_local_byte));
            self.emit_byte(OpCode::Constant(upvalue.index));
        }
    }
    fn method(&mut self) {
        self.consume(TokenType::TokenIdentifier, "Expect method name.".to_owned());
        let constant = self.identifier_constant_once(&self.previous.clone());
        let mut type_ = FunctionType::TypeMethod;
        if self.previous.lexeme == "init" {
            type_ = FunctionType::TypeInitializer;
        }
        self.function(type_);
        self.emit_byte(OpCode::Method(constant));
    }
    fn function_declaration(&mut self) {
        self.consume(
            TokenType::TokenIdentifier,
            "Expect function name".to_owned(),
        );
        let name_token = self.previous.clone();
        self.declare_variable();
        self.mark_initialized();

        self.function(FunctionType::TypeFunction);

        if self.scope_depth == 0 {
            let global_index = self.identifier_constant_once(&name_token);
            self.emit_byte(OpCode::DefineGlobal(global_index));
        }
    }
    fn var_declaration(&mut self) {
        self.consume(
            TokenType::TokenIdentifier,
            "Expect variable name.".to_owned(),
        );
        let name_token = self.previous.clone();
        self.declare_variable();

        if self.match_token(TokenType::TokenEqual) {
            self.expression();
        } else {
            self.emit_byte(OpCode::OpNil);
        }

        self.consume(
            TokenType::TokenSemicolon,
            "Expect ';' after variable declaration.".to_owned(),
        );

        if self.scope_depth == 0 {
            let global_index = self.identifier_constant_once(&name_token);
            self.emit_byte(OpCode::DefineGlobal(global_index));
        } else {
            self.mark_initialized();
        }
    }
    fn define_variable_by_name(&mut self, name_token: &Token) {
        if self.scope_depth > 0 {
            self.mark_initialized();
            return;
        }
        let name_constant = self.identifier_constant_once(&name_token.clone());
        self.emit_byte(OpCode::DefineGlobal(name_constant));
    }
    fn class_declaration(&mut self) {
        self.consume(TokenType::TokenIdentifier, "Expect class name.".to_owned());
        let class_name = self.previous.clone();
        let name_constant = self.identifier_constant_once(&self.previous.clone());
        self.declare_variable();
        self.emit_byte(OpCode::Class(name_constant));
        self.define_variable(name_constant);
        let class_compiler = ClassCompiler {
            enclosing: self.class_compiler.clone(),
            has_super_class: false,
        };
        self.class_compiler = Some(Box::new(class_compiler.clone()));
        if self.match_token(TokenType::TokenLess) {
            self.consume(
                TokenType::TokenIdentifier,
                "Expected superclass name.".to_owned(),
            );
            self.variable(false);
            if class_name == self.previous {
                self.error("A class can't ingerit from itself".to_owned());
            }
            self.begin_scope();
            self.define_variable(0);
            self.named_variable(class_name.clone(), false);
            self.emit_byte(OpCode::OpInherit);
            let mut class_compiler_tmp = self.class_compiler.clone().unwrap();
            class_compiler_tmp.has_super_class = true;
            self.class_compiler = Some(class_compiler_tmp.clone());
        }
        self.named_variable(class_name.clone(), false);
        self.consume(
            TokenType::TokenLeftBrace,
            "Expect '{' before class body.".to_owned(),
        );
        while !self.check(TokenType::TokenRightBrace) && !self.check(TokenType::TokenEof) {
            self.method();
        }
        self.consume(
            TokenType::TokenRightBrace,
            "Expect '}' after class body.".to_owned(),
        );
        self.emit_byte(OpCode::OpPop);
        if self.class_compiler.clone().unwrap().has_super_class {
            self.end_scope();
        }
        self.class_compiler = self.class_compiler.clone().unwrap().enclosing;
    }
    fn declaration(&mut self) {
        if self.match_token(TokenType::TokenClass) {
            self.class_declaration();
        } else if self.match_token(TokenType::TokenFun) {
            self.function_declaration();
        } else if self.match_token(TokenType::TokenVar) {
            self.var_declaration();
        } else {
            self.statement();
        }
        if self.panic_mode {
            self.synchronize()
        }
    }
    fn block(&mut self) {
        while !self.check(TokenType::TokenRightBrace) && !self.check(TokenType::TokenEof) {
            self.declaration();
        }
        self.consume(
            TokenType::TokenRightBrace,
            "Expect '}' after block.".to_owned(),
        );
    }
    fn compile(&mut self, source: String) -> ObjFunction {
        let scanner = Scanner::new(source);
        self.scanner = Some(scanner);
        self.advance();
        while !self.match_token(TokenType::TokenEof) {
            self.declaration();
        }
        let function = self.end_compiler();
        function
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ObjFunction;

    #[test]
    fn test_expression1() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::Constant(1),
                OpCode::OpAdd,
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 1, 1, 1, 1],
            constants: vec![Value::Number(2f64), Value::Number(3f64)],
            count: 6,
        };
        let source = "2 + 3;".to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();
        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_expression2() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::Constant(1),
                OpCode::OpAdd,
                OpCode::DefineGlobal(2),
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 1, 1, 1, 1],
            constants: vec![
                Value::Number(2f64),
                Value::Number(3f64),
                Value::String("x".to_owned()),
            ],
            count: 6,
        };
        let source = "var x = 2 + 3;".to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();
        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_statement() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::OpPrint,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 1, 1],
            constants: vec!["test".into()],
            count: 4,
        };
        let source = "print \"test\";".to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();
        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_print_variable() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::DefineGlobal(1),
                OpCode::GetGlobal(1),
                OpCode::OpPrint,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 1, 1, 1, 1],
            constants: vec![
                Value::String("test".to_owned()),
                Value::String("x".to_owned()),
            ],
            count: 6,
        };

        let source = "var x = \"test\"; print x;".to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_function() {
        let expected_inner_chunk = Chunk {
            code: vec![
                OpCode::Constant(0), // "2"
                OpCode::GetLocal(1), // x
                OpCode::OpAdd,
                OpCode::GetLocal(2), // y
                OpCode::OpReturn,
            ],
            lines: vec![2, 2, 2, 3, 3],
            constants: vec![Value::Number(2f64)],
            count: 5,
        };
        let mut internal_fn = ObjFunction::new();
        internal_fn.name = "test".to_string();
        internal_fn.arity = 1;
        internal_fn.chunk = expected_inner_chunk.clone();

        let mut expected_chunk = Chunk {
            code: vec![
                // fun test(x) { ... }
                OpCode::Closure(0),
                OpCode::DefineGlobal(1),
                // print test(10);
                OpCode::GetGlobal(1),
                OpCode::Constant(2),
                OpCode::Call(1),
                OpCode::OpPrint,
                // implicit script return
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![4, 4, 5, 5, 5, 5, 5, 5],
            constants: vec![],
            count: 8,
        };
        expected_chunk.constants = vec![
            Value::Function(Box::new(internal_fn)),
            Value::String("test".into()),
            Value::Number(10f64),
        ];

        let source = r#"fun test(x) {
                                var y = 2 + x;
                                return y;
                            }
                            print test(10);"#
            .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        let script_fn = compiler.compile(source);
        let chunk = compiler.current_chunk();

        let test_fn_value = &script_fn.chunk.constants[0];
        let test_fn = test_fn_value.as_function();
        assert_eq!(test_fn.name, "test");
        assert_eq!(test_fn.arity, 1);
        assert_eq!(test_fn.chunk, expected_inner_chunk);

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_block() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::DefineGlobal(1),
                OpCode::Constant(2),
                OpCode::GetLocal(1),
                OpCode::GetGlobal(1),
                OpCode::OpAdd,
                OpCode::SetLocal(1),
                OpCode::OpPop,
                OpCode::GetLocal(1),
                OpCode::OpPrint,
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![2, 2, 4, 5, 5, 5, 5, 5, 6, 6, 7, 8, 8],
            constants: vec![
                Value::Number(10f64),
                Value::String("x".to_owned()),
                Value::Number(2f64),
            ],
            count: 13,
        };

        let source = r#"
                            var x = 10
                            {
                                var y = 2;
                                y = y + x;
                                print y;
                            }
                           "#
        .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_class() {
        let expected_init_chunk = Chunk {
            count: 5,
            code: vec![
                OpCode::GetLocal(1),
                OpCode::SetProperty(0),
                OpCode::OpPop,
                OpCode::GetLocal(0),
                OpCode::OpReturn,
            ],
            lines: vec![3, 3, 3, 4, 4],
            constants: vec![Value::String("test".to_owned())],
        };

        let mut expected_init_fn = ObjFunction::new();
        expected_init_fn.name = "init".to_owned();
        expected_init_fn.arity = 1;
        expected_init_fn.upvalue_count = 0;
        expected_init_fn.chunk = expected_init_chunk;

        let expected_do_staff_chunk = Chunk {
            count: 6,
            code: vec![
                OpCode::GetProperty(0),
                OpCode::GetLocal(1),
                OpCode::OpAdd,
                OpCode::OpPrint,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![6, 6, 6, 6, 7, 7],
            constants: vec![Value::String("test".to_owned())],
        };

        let mut expected_do_staff_fn = ObjFunction::new();
        expected_do_staff_fn.name = "doStaff".to_owned();
        expected_do_staff_fn.arity = 1;
        expected_do_staff_fn.upvalue_count = 0;
        expected_do_staff_fn.chunk = expected_do_staff_chunk;

        let expected_chunk = Chunk {
            count: 18,
            code: vec![
                OpCode::Class(0),
                OpCode::DefineGlobal(0),
                OpCode::GetGlobal(0),
                OpCode::Closure(2),
                OpCode::Method(1),
                OpCode::Closure(4),
                OpCode::Method(3),
                OpCode::OpPop,
                OpCode::GetGlobal(0),
                OpCode::Constant(5),
                OpCode::Call(1),
                OpCode::DefineGlobal(6),
                OpCode::GetGlobal(6),
                OpCode::Constant(7),
                OpCode::Invoke(3, 1),
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 1, 4, 4, 7, 7, 8, 9, 9, 9, 9, 10, 10, 10, 10, 10, 10],
            constants: vec![
                Value::String("TestClass".to_owned()),
                Value::String("init".to_owned()),
                Value::Function(Box::new(expected_init_fn)),
                Value::String("doStaff".to_owned()),
                Value::Function(Box::new(expected_do_staff_fn)),
                Value::Number(2f64),
                Value::String("obj".to_owned()),
                Value::Number(4f64),
            ],
        };

        let source = r#"class TestClass {
                                init(x) {
                                  this.test=x;
                                }
                                doStaff(y) {
                                   print this.test + y;
                                }
                            }
                          var obj = TestClass(2);
                          obj.doStaff(4);"#
            .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_for() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::GetLocal(1),
                OpCode::Constant(1),
                OpCode::OpLess,
                OpCode::JumpIfFalse(11),
                OpCode::OpPop,
                OpCode::Jump(6),
                OpCode::GetLocal(1),
                OpCode::Constant(2),
                OpCode::OpAdd,
                OpCode::SetLocal(1),
                OpCode::OpPop,
                OpCode::Loop(12),
                OpCode::GetLocal(1),
                OpCode::OpPrint,
                OpCode::Loop(9),
                OpCode::OpPop,
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 3, 3, 3, 3, 3],
            constants: vec![
                Value::Number(1f64),
                Value::Number(5f64),
                Value::Number(1f64),
            ],
            count: 20,
        };

        let source = r#"for (var i = 1; i < 5; i = i + 1) {
                             print i;
                            }"#
        .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_while() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::DefineGlobal(1),
                OpCode::GetGlobal(1),
                OpCode::Constant(2),
                OpCode::OpLess,
                OpCode::JumpIfFalse(9),
                OpCode::OpPop,
                OpCode::GetGlobal(1),
                OpCode::OpPrint,
                OpCode::GetGlobal(1),
                OpCode::Constant(3),
                OpCode::OpAdd,
                OpCode::SetGlobal(1),
                OpCode::OpPop,
                OpCode::Loop(13),
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 2, 2, 2, 2, 2, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5],
            constants: vec![
                Value::Number(0f64),
                Value::String("i".to_owned()),
                Value::Number(5f64),
                Value::Number(1f64),
            ],
            count: 18,
        };

        let source = r#"var i = 0;
                            while (i < 5) {
                                print i;
                                i = i +1;
                            }"#
        .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_while_block() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::GetLocal(1),
                OpCode::Constant(1),
                OpCode::OpLess,
                OpCode::JumpIfFalse(9),
                OpCode::OpPop,
                OpCode::GetLocal(1),
                OpCode::OpPrint,
                OpCode::GetLocal(1),
                OpCode::Constant(2),
                OpCode::OpAdd,
                OpCode::SetLocal(1),
                OpCode::OpPop,
                OpCode::Loop(13),
                OpCode::OpPop,
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![2, 3, 3, 3, 3, 3, 4, 4, 5, 5, 5, 5, 5, 6, 6, 7, 7, 7],
            constants: vec![
                Value::Number(0f64),
                Value::Number(5f64),
                Value::Number(1f64),
            ],
            count: 18,
        };

        let source = r#"{
                                var i = 0;
                                while (i < 5) {
                                    print i;
                                    i = i +1;
                                }
                            }"#
        .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_if() {
        let expected_chunk = Chunk {
            code: vec![
                OpCode::Constant(0),
                OpCode::DefineGlobal(1),
                OpCode::Constant(2),
                OpCode::DefineGlobal(3),
                OpCode::GetGlobal(1),
                OpCode::GetGlobal(3),
                OpCode::OpLess,
                OpCode::JumpIfFalse(4),
                OpCode::OpPop,
                OpCode::Constant(4),
                OpCode::OpPrint,
                OpCode::Jump(3),
                OpCode::OpPop,
                OpCode::Constant(5),
                OpCode::OpPrint,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![1, 1, 2, 2, 3, 3, 3, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7],
            constants: vec![
                Value::Number(2f64),
                Value::String("a".to_owned()),
                Value::Number(3f64),
                Value::String("b".to_owned()),
                Value::String("a is less than b".to_owned()),
                Value::String("a is greater than b".to_owned()),
            ],
            count: 17,
        };

        let source = r#"var a = 2;
                            var b = 3;
                            if (a < b) {
                              print("a is less than b");
                            } else {
                            print("a is greater than b");
                            }"#
        .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_closure1() {
        let expected_inner_fun2_chunk = Chunk {
            count: 4,
            code: vec![
                OpCode::GetUpvalue(0),
                OpCode::Constant(0),
                OpCode::OpAdd,
                OpCode::OpReturn,
            ],
            lines: vec![4, 4, 4, 4],
            constants: vec![Value::Number(2f64)],
        };

        let mut expected_fun2 = ObjFunction::new();
        expected_fun2.name = "fun2".to_owned();
        expected_fun2.arity = 0;
        expected_fun2.upvalue_count = 1;
        expected_fun2.chunk = expected_inner_fun2_chunk;

        let expected_inner_fun1_chunk = Chunk {
            count: 8,
            code: vec![
                OpCode::GetLocal(1),
                OpCode::Constant(0),
                OpCode::OpAdd,
                OpCode::Closure(1),
                OpCode::Constant(1),
                OpCode::Constant(2),
                OpCode::GetLocal(3),
                OpCode::OpReturn,
            ],
            lines: vec![2, 2, 2, 5, 5, 5, 6, 6],
            constants: vec![
                Value::Number(1f64),
                Value::Function(Box::new(expected_fun2)),
            ],
        };

        let mut expected_fun1 = ObjFunction::new();
        expected_fun1.name = "fun1".to_owned();
        expected_fun1.arity = 1;
        expected_fun1.upvalue_count = 0;
        expected_fun1.chunk = expected_inner_fun1_chunk;

        let expected_chunk = Chunk {
            count: 11,
            code: vec![
                OpCode::Closure(0),
                OpCode::DefineGlobal(1),
                OpCode::GetGlobal(1),
                OpCode::Constant(2),
                OpCode::Call(1),
                OpCode::DefineGlobal(3),
                OpCode::GetGlobal(3),
                OpCode::Call(0),
                OpCode::OpPrint,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![7, 7, 8, 8, 8, 8, 9, 9, 9, 9, 9],
            constants: vec![
                Value::Function(Box::new(expected_fun1)),
                Value::String("fun1".to_owned()),
                Value::Number(10f64),
                Value::String("c".to_owned()),
            ],
        };

        let source = r#"fun fun1(x) {
                                var y = x + 1;
                                fun fun2() {
                                    return y + 2;
                                }
                                return fun2;
                            }
                            var c = fun1(10);
                            print c();"#
            .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_closure2() {
        let expected_inner_fun2_chunk = Chunk {
            count: 7,
            code: vec![
                OpCode::GetUpvalue(0),
                OpCode::GetUpvalue(1),
                OpCode::OpAdd,
                OpCode::GetLocal(1),
                OpCode::GetUpvalue(2),
                OpCode::OpAdd,
                OpCode::OpReturn,
            ],
            lines: vec![5, 5, 5, 6, 6, 6, 6],
            constants: vec![],
        };

        let mut expected_fun2 = ObjFunction::new();
        expected_fun2.name = "fun2".to_owned();
        expected_fun2.arity = 0;
        expected_fun2.upvalue_count = 3;
        expected_fun2.chunk = expected_inner_fun2_chunk;

        let expected_inner_fun1_chunk = Chunk {
            count: 13,
            code: vec![
                OpCode::GetLocal(1),
                OpCode::Constant(0),
                OpCode::OpAdd,
                OpCode::Constant(1),
                OpCode::Closure(2),
                OpCode::Constant(1),
                OpCode::Constant(2),
                OpCode::Constant(1),
                OpCode::Constant(3),
                OpCode::Constant(1),
                OpCode::Constant(1),
                OpCode::GetLocal(4),
                OpCode::OpReturn,
            ],
            lines: vec![2, 2, 2, 3, 7, 7, 7, 7, 7, 7, 7, 8, 8],
            constants: vec![
                Value::Number(1f64),
                Value::Number(10f64),
                Value::Function(Box::new(expected_fun2)),
            ],
        };

        let mut expected_fun1 = ObjFunction::new();
        expected_fun1.name = "fun1".to_owned();
        expected_fun1.arity = 1;
        expected_fun1.upvalue_count = 0;
        expected_fun1.chunk = expected_inner_fun1_chunk;

        let expected_chunk = Chunk {
            count: 11,
            code: vec![
                OpCode::Closure(0),
                OpCode::DefineGlobal(1),
                OpCode::GetGlobal(1),
                OpCode::Constant(2),
                OpCode::Call(1),
                OpCode::DefineGlobal(3),
                OpCode::GetGlobal(3),
                OpCode::Call(0),
                OpCode::OpPrint,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![9, 9, 10, 10, 10, 10, 11, 11, 11, 11, 11],
            constants: vec![
                Value::Function(Box::new(expected_fun1)),
                Value::String("fun1".to_owned()),
                Value::Number(10f64),
                Value::String("c".to_owned()),
            ],
        };

        let source = r#"fun fun1(x) {
                                var y = x + 1;
                                var z = 10;
                                fun fun2() {
                                    var j = y + z;
                                    return j + x;
                                }
                                return fun2;
                            }
                            var c = fun1(10);
                            print c();"#
            .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_closure3() {
        let expected_inner_fun2_chunk = Chunk {
            count: 4,
            code: vec![
                OpCode::Constant(0),
                OpCode::Constant(1),
                OpCode::OpAdd,
                OpCode::OpReturn,
            ],
            lines: vec![3, 3, 3, 3],
            constants: vec![Value::Number(2f64), Value::Number(2f64)],
        };

        let mut expected_fun2 = ObjFunction::new();
        expected_fun2.name = "fun2".to_owned();
        expected_fun2.arity = 0;
        expected_fun2.upvalue_count = 0;
        expected_fun2.chunk = expected_inner_fun2_chunk;

        let expected_inner_fun1_chunk = Chunk {
            count: 3,
            code: vec![OpCode::Closure(0), OpCode::GetLocal(1), OpCode::OpReturn],
            lines: vec![4, 5, 5],
            constants: vec![Value::Function(Box::new(expected_fun2))],
        };

        let mut expected_fun1 = ObjFunction::new();
        expected_fun1.name = "fun1".to_owned();
        expected_fun1.arity = 0;
        expected_fun1.upvalue_count = 0;
        expected_fun1.chunk = expected_inner_fun1_chunk;

        let expected_chunk = Chunk {
            count: 10,
            code: vec![
                OpCode::Closure(0),
                OpCode::DefineGlobal(1),
                OpCode::GetGlobal(1),
                OpCode::Call(0),
                OpCode::DefineGlobal(2),
                OpCode::GetGlobal(2),
                OpCode::Call(0),
                OpCode::OpPrint,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![6, 6, 7, 7, 7, 8, 8, 8, 8, 8],
            constants: vec![
                Value::Function(Box::new(expected_fun1)),
                Value::String("fun1".to_owned()),
                Value::String("c".to_owned()),
            ],
        };

        let source = r#"fun fun1() {
                                fun fun2() {
                                    return 2 + 2;
                                }
                                return fun2;
                            }
                            var c = fun1();
                            print c();"#
            .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
    #[test]
    fn test_recursion() {
        let expected_inner_test_chunk = Chunk {
            count: 17,
            code: vec![
                OpCode::GetLocal(1),
                OpCode::Constant(0),
                OpCode::OpGreater,
                OpCode::JumpIfFalse(10),
                OpCode::OpPop,
                OpCode::GetGlobal(1),
                OpCode::GetLocal(1),
                OpCode::Constant(2),
                OpCode::OpSubtract,
                OpCode::Call(1),
                OpCode::OpPop,
                OpCode::Constant(3),
                OpCode::OpPrint,
                OpCode::Jump(1),
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 5, 5, 6, 6],
            constants: vec![
                Value::Number(0f64),
                Value::String("test".to_owned()),
                Value::Number(1f64),
                Value::Number(1f64),
            ],
        };

        let mut expected_test_fn = ObjFunction::new();
        expected_test_fn.name = "test".to_owned();
        expected_test_fn.arity = 1;
        expected_test_fn.upvalue_count = 0;
        expected_test_fn.chunk = expected_inner_test_chunk;

        let expected_chunk = Chunk {
            count: 8,
            code: vec![
                OpCode::Closure(0),
                OpCode::DefineGlobal(1),
                OpCode::GetGlobal(1),
                OpCode::Constant(2),
                OpCode::Call(1),
                OpCode::OpPop,
                OpCode::OpNil,
                OpCode::OpReturn,
            ],
            lines: vec![6, 6, 7, 7, 7, 7, 7, 7],
            constants: vec![
                Value::Function(Box::new(expected_test_fn)),
                Value::String("test".to_owned()),
                Value::Number(4f64),
            ],
        };

        let source = r#"fun test(i) {
                            if (i > 0) {
                                test(i - 1);
                                print 1;
                              }
                            }
                            test(4);"#
            .to_owned();
        let mut compiler = Compiler::new(None, FunctionType::TypeScript);
        compiler.compile(source);
        let chunk = compiler.current_chunk();

        assert_eq!(chunk, &expected_chunk);
    }
}
// TODO add tests:
// a closure that captures a local from two levels up (upvalue-of-upvalue case)
// Short-circuit logic correctness
// Error handling
// inheritance
