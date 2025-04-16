from src.token_ import Token, TokenType


class LexicalError(Exception):
    def __init__(self, message):
        super().__init__(message)


class Scanner:
    keywords: dict[str, TokenType] = {
        "and": TokenType.AND,
        "or": TokenType.OR,
        "class": TokenType.CLASS,
        "else": TokenType.ELSE,
        "false": TokenType.FALSE,
        "true": TokenType.TRUE,
        "for": TokenType.FOR,
        "fun": TokenType.FUN,
        "if": TokenType.IF,
        "nil": TokenType.NIL,
        "print": TokenType.PRINT,
        "return": TokenType.RETURN,
        "super": TokenType.SUPER,
        "this": TokenType.THIS,
        "var": TokenType.VAR,
        "while": TokenType.WHILE,
    }

    def __init__(self, source: str):
        self.source = source
        self.tokens: list[Token] = []

        self.start = 0
        self.current = 0
        self.line = 1
        self.had_error = False

    def report_error_unexpected_character(self, char: str, line: int):
        self.had_error = True
        print(f"Unexpected character: {char} in line: {line}")

    def report_error_unterminated_string(self, line: int):
        self.had_error = True
        print(f"Undetermined string in line: {line}")

    def is_at_end(self) -> bool:
        return self.current >= len(self.source)

    def peek(self) -> str:
        if self.is_at_end():
            return "\0"
        return self.source[self.current]

    def peek_next(self) -> str:
        if self.current + 1 >= len(self.source):
            return "\0"
        return self.source[self.current + 1]

    def advance(self):
        next_char = self.source[self.current]
        self.current += 1
        return next_char

    def add_token(self, token_type: TokenType, literal: str | float | None = None):
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

    def string(self):
        while self.peek() != '"' and not self.is_at_end():
            if self.peek() == "\n":
                self.line += 1
            self.advance()
        if self.is_at_end():
            self.report_error_unterminated_string(line=self.line)
        self.advance()
        literal = self.source[self.start + 1 : self.current - 1]
        self.add_token(token_type=TokenType.STRING, literal=literal)

    def number(self):
        while self.peek().isdigit():
            self.advance()
        if self.peek() == "." and self.peek_next().isdigit():
            self.advance()
        while self.peek().isdigit():
            self.advance()
        self.add_token(
            TokenType.NUMBER, literal=float(self.source[self.start : self.current])
        )

    def identifier(self):
        while self.peek().isalnum():
            self.advance()
        text = self.source[self.start : self.current]
        token_type = self.keywords.get(text, None)
        if token_type:
            self.add_token(token_type=token_type)
        else:
            self.add_token(token_type=TokenType.IDENTIFIER)

    def one_line_comment(self):
        while self.peek() != "\n" and not self.is_at_end():
            self.advance()
        self.add_token(
            token_type=TokenType.COMMENT, literal=self.source[self.start : self.current]
        )

    def multi_line_comment(self):
        while self.peek() != "*" and self.peek_next() != "/" and not self.is_at_end():
            if self.match("\n"):
                self.line += 1
            self.advance()
        self.advance()
        self.advance()  # handle last 2 characters */
        self.add_token(
            token_type=TokenType.COMMENT, literal=self.source[self.start : self.current]
        )

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
                    self.one_line_comment()
                elif self.match("*"):
                    self.multi_line_comment()
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
            case '"':
                self.string()
            case _:
                if char.isdigit():
                    self.number()
                elif char.isalpha():
                    self.identifier()
                else:
                    self.report_error_unexpected_character(char=char, line=self.line)

    def scan_tokens(self) -> list[Token]:
        while not self.is_at_end():
            self.start = self.current
            self.scan_token()
        self.tokens.append(
            Token(token_type=TokenType.EOF, lexeme="", literal=None, line=self.line)
        )
        return self.tokens
