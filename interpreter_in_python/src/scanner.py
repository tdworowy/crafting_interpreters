from enum import Enum, auto


class LexicalError(Exception):
    def __init__(self, message):
        super().__init__(message)


class TokenType(Enum):
    # Single-character tokens.
    LEFT_PAREN = auto()
    RIGHT_PAREN = auto()
    LEFT_BRACE = auto()
    RIGHT_BRACE = auto()
    COMMA = auto()
    DOT = auto()
    MINUS = auto()
    PLUS = auto()
    SEMICOLON = auto()
    SLASH = auto()
    STAR = auto()

    # One or two character tokens.
    BANG = auto()
    BANG_EQUAL = auto()
    EQUAL = auto()
    EQUAL_EQUAL = auto()
    GREATER = auto()
    GREATER_EQUAL = auto()
    LESS = auto()
    LESS_EQUAL = auto()

    # Literals.
    IDENTIFIER = auto()
    STRING = auto()
    NUMBER = auto()

    # Keywords.
    AND = auto()
    CLASS = auto()
    ELSE = auto()
    FALSE = auto()
    FUN = auto()
    FOR = auto()
    IF = auto()
    NIL = auto()
    OR = auto()
    PRINT = auto()
    RETURN = auto()
    SUPER = auto()
    THIS = auto()
    TRUE = auto()
    VAR = auto()
    WHILE = auto()

    EOF = auto()


class Token:
    def __init__(
        self, token_type: TokenType, lexeme: str, literal: str | None, line: int
    ):
        self.token_type = token_type
        self.lexeme = lexeme
        self.literal = literal
        self.line = line

    def __str__(self):
        return f"{self.token_type} {self.lexeme} {self.literal}"


class Scanner:
    keywords: dict[str, Token]

    def __init__(self, source: str):
        self.source = source
        self.tokens: list[Token] = []

        self.start = 0
        self.current = 0
        self.line = 1
        self.had_error = False

    def report_error(self, char: str, line: int):
        self.had_error = True
        print(f"Unexpected character: {char} in line: {line}")

    def is_at_end(self) -> bool:
        return self.current >= len(self.source)

    def peek(self) -> str:
        if self.is_at_end():
            return "\0"
        return self.source[self.current]

    def advance(self):
        next_char = self.source[self.current]
        self.current += 1
        return next_char

    def add_token(self, token_type: TokenType, literal: str | None = None):
        text = self.source[self.start : self.current]
        self.tokens.append(
            Token(token_type=token_type, lexeme=text, literal=literal, line=self.line)
        )

    def match(self, expected: str) -> bool:
        if self.is_at_end():
            return False
        if self.source[self.current] != expected:
            return False
        self.current += 1
        return True

    def scan_token(self):
        char = self.advance()
        match char:
            case "(":
                self.add_token(token_type=TokenType.LEFT_PAREN)
            case ")":
                self.add_token(token_type=TokenType.RIGHT_PAREN)
            case "{":
                self.add_token(token_type=TokenType.LEFT_BRACE)
            case "}":
                self.add_token(token_type=TokenType.RIGHT_BRACE)
            case ",":
                self.add_token(token_type=TokenType.COMMA)
            case ".":
                self.add_token(token_type=TokenType.DOT)
            case "-":
                self.add_token(token_type=TokenType.MINUS)
            case "+":
                self.add_token(token_type=TokenType.PLUS)
            case ";":
                self.add_token(token_type=TokenType.SEMICOLON)
            case "*":
                self.add_token(token_type=TokenType.STAR)
            case "!":
                self.add_token(
                    token_type=(
                        TokenType.BANG_EQUAL if self.match("=") else TokenType.BANG
                    )
                )
            case "=":
                self.add_token(
                    token_type=(
                        TokenType.EQUAL_EQUAL if self.match("=") else TokenType.EQUAL
                    )
                )
            case "<":
                self.add_token(
                    token_type=(
                        TokenType.LESS_EQUAL if self.match("=") else TokenType.LESS
                    )
                )
            case ">":
                self.add_token(
                    token_type=(
                        TokenType.GREATER_EQUAL
                        if self.match("=")
                        else TokenType.GREATER
                    )
                )
            case "/":
                if self.match("/"):
                    while self.peek() != "\n" and not self.is_at_end():
                        self.advance()
                else:
                    self.add_token(token_type=TokenType.SLASH)
            case " ":
                pass
            case "\r":
                pass
            case "\t":
                pass
            case "\n":
                self.line += 1
            case _:
                self.report_error(char=char, line=self.line)

    def scan_tokens(self) -> list[Token]:
        while not self.is_at_end():
            self.start = self.current
            self.scan_token()
        self.tokens.append(
            Token(token_type=TokenType.EOF, lexeme="", literal=None, line=self.line)
        )
        return self.tokens
