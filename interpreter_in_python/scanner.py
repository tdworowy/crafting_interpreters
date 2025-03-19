from enum import Enum, auto


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
    LESS, LESS_EQUAL = auto()

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


class Scaner:
    keywords: dict[str, Token]

    def __init__(self, source: str):
        self.source = source
        self.tokens: list[Token] = []

        self.start = 0
        self.current = 0
        self.line = 1

    def is_at_end(self) -> bool:
        return self.current >= len(self.source)

    def advance(self):
        return self.source[self.current + 1]

    def add_token(self, token_type: TokenType, literal: str | None = None):
        text = self.source[self.start : self.current]
        self.tokens.append(
            Token(token_type=token_type, lexeme=text, literal=literal, line=self.line)
        )

    def scan_token(self):
        match self.advance():
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

    def scan_tokens(self) -> list[Token]:
        while not self.is_at_end():
            self.start = self.current
            self.scan_token()
        self.tokens.append(
            Token(token_type=TokenType.EOF, lexeme="", literal=None, line=self.line)
        )
        return self.tokens
