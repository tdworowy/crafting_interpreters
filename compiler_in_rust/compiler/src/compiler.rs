use crate::{
    chunks::Chunk,
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
#[derive(Copy, Clone)]
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

struct Local {
    name: Token,
    depth: usize,
    is_captured: bool,
}

struct Upvalue {
    index: usize,
    is_local: bool,
}

enum FunctionType {
    TYPE_FUNCTION,
    TYPE_SCRIPT,
    TYPE_METHOD,
    TYPE_INITIALIZER,
}

struct Compiler<'a> {
    enclosing: &'a Compiler<'a>,
    function: &'a ObjFunction<'a>,
    function_type: FunctionType,
    locals: Vec<Local>,
    upvalues: Vec<Upvalue>,
    scope_depth: usize,
}

struct ClassCompiler<'a> {
    enclosing: &'a ClassCompiler<'a>,
    has_super_class: bool,
}

impl<'a> Compiler<'a> {
    fn current_chunk(&self) -> &Chunk {
        return &self.function.chunk;
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
}
