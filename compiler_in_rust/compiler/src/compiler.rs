use std::collections::HashMap;

use crate::{
    chunks::{Chunk, OpCode},
    object::ObjFunction,
    scaner::{Scanner, Token, TokenType},
};

struct Parser {
    scanner: Scanner,
    current: Token,
    previous: Token,
    had_error: bool,
    panic_mode: bool,
}
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

type ParseFn = fn(can_assign: bool);

#[derive(Copy, Clone)]
struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
    pub precedence: Precedence,
}

lazy_static::lazy_static! {
    static ref RULES: HashMap<TokenType, ParseRule> = {
        let mut m = HashMap::new();

        m.insert(TokenType::TOKEN_LEFT_PAREN,     ParseRule { prefix: Some(grouping), infix: Some(call),     precedence: Precedence::PREC_CALL });
        m.insert(TokenType::TOKEN_RIGHT_PAREN,    ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_LEFT_BRACE,     ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_RIGHT_BRACE,    ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_DOT,            ParseRule { prefix: None,         infix: Some(dot),      precedence: Precedence::PREC_CALL });
        m.insert(TokenType::TOKEN_MINUS,          ParseRule { prefix: Some(unary),  infix: Some(binary),   precedence: Precedence::PREC_TERM });
        m.insert(TokenType::TOKEN_PLUS,           ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_TERM });
        m.insert(TokenType::TOKEN_SEMICOLON,      ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_SLASH,          ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_FACTOR });
        m.insert(TokenType::TOKEN_STAR,           ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_FACTOR });
        m.insert(TokenType::TOKEN_BANG,           ParseRule { prefix: Some(unary),  infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_BANG_EQUAL,     ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_EQUALITY });
        m.insert(TokenType::TOKEN_EQUAL,          ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_EQUAL_EQUAL,    ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_EQUALITY });
        m.insert(TokenType::TOKEN_GREATER,        ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_GREATER_EQUAL,  ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_LESS,           ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_LESS_EQUAL,     ParseRule { prefix: None,         infix: Some(binary),   precedence: Precedence::PREC_COMPARISON });
        m.insert(TokenType::TOKEN_IDENTIFIER,     ParseRule { prefix: Some(variable), infix: None,         precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_STRING,         ParseRule { prefix: Some(string), infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_NUMBER,         ParseRule { prefix: Some(number), infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_AND,            ParseRule { prefix: None,         infix: Some(and_),     precedence: Precedence::PREC_AND });
        m.insert(TokenType::TOKEN_CLASS,          ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_ELSE,           ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_FALSE,          ParseRule { prefix: Some(literal), infix: None,          precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_FOR,            ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_FUN,            ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_IF,             ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_NIL,            ParseRule { prefix: Some(literal), infix: None,          precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_OR,             ParseRule { prefix: None,         infix: Some(or_),      precedence: Precedence::PREC_OR });
        m.insert(TokenType::TOKEN_PRINT,          ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_RETURN,         ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_SUPER,          ParseRule { prefix: Some(super_), infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_THIS,           ParseRule { prefix: Some(this_),  infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_TRUE,           ParseRule { prefix: Some(literal), infix: None,          precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_VAR,            ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_WHILE,          ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_ERROR,          ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });
        m.insert(TokenType::TOKEN_EOF,            ParseRule { prefix: None,         infix: None,           precedence: Precedence::PREC_NONE });

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

struct Local {
    name: Token,
    depth: usize,
    is_captured: bool,
}

struct Upvalue {
    index: usize,
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
    function: Box<ObjFunction<'a>>,
    function_type: FunctionType,
    locals: Vec<Local>,
    upvalues: Vec<Upvalue>,
    scope_depth: usize,
    parser: Parser,
}

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
        let line = self.parser.previous.line;
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
    fn emit_return(&mut self) {
        if self.function_type == FunctionType::TYPE_INITIALIZER {
            self.emit_bytes(OpCode::OP_GET_LOCAL as u8, 0);
        } else {
            self.emit_byte(OpCode::OP_NIL as u8);
        }
        self.emit_byte(OpCode::OP_RETURN as u8);
    }
}

impl Parser {
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
        let prefix_rule: ParseFn = match get_rule(self.previous.token_type.clone()).prefix {
            Some(rule) => rule,
            _ => {
                self.error("Expect expression.".to_owned());
                return;
            }
        };
        let can_assign = precedence <= Precedence::PREC_ASSIGNMENT;
        prefix_rule(can_assign);
        while precedence <= get_rule(self.current.token_type.clone()).precedence {
            self.advance();
            let infix_rule: ParseFn = match get_rule(self.previous.token_type.clone()).infix {
                Some(rule) => rule,
                _ => {
                    return;
                }
            };
            infix_rule(can_assign);
            if can_assign && self.match_token(TokenType::TOKEN_EQUAL) {
                self.error("Invalid assigment target.".to_owned());
            }
        }
    }
}
