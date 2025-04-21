from src.expr import Expr, Binary, Unary, Literal, Grouping
from src.token_ import Token, TokenType


class PassingError(Exception):
    def __init__(self, message):
        super().__init__(message)


class Parser:
    def __init__(self, tokens: list[Token]):
        self.tokens = tokens
        self.current = 0

    def expression(self) -> Expr:
        return self.equality()

    def equality(self) -> Expr:
        expr = self.comparison()
        while self.match([TokenType.BANG_EQUAL, TokenType.EQUAL_EQUAL]):
            operator = self.previous()
            right = self.comparison()
            expr = Binary(left=expr, operator=operator, right=right)
        return expr

    def comparison(self) -> Expr:
        expr = self.term()
        while self.match(
            [
                TokenType.GREATER,
                TokenType.GREATER_EQUAL,
                TokenType.LESS,
                TokenType.LESS_EQUAL,
            ]
        ):
            operator = self.previous()
            right = self.term()
            expr = Binary(left=expr, operator=operator, right=right)
        return expr

    def term(self) -> Expr:
        expr = self.factor()
        while self.match([TokenType.MINUS, TokenType.PLUS]):
            operator = self.previous()
            right = self.factor()
            expr = Binary(left=expr, operator=operator, right=right)
        return expr

    def factor(self) -> Expr:
        expr = self.unary()
        while self.match([TokenType.SLASH, TokenType.STAR]):
            operator = self.previous()
            right = self.unary()
            expr = Binary(left=expr, operator=operator, right=right)
        return expr

    def unary(self) -> Expr:
        if self.match([TokenType.BANG, TokenType.MINUS]):
            operator = self.previous()
            right = self.unary()
            return Unary(operator=operator, right=right)
        return self.primary()

    def primary(self) -> Expr:
        if self.match([TokenType.FALSE]):
            return Literal(value=False)
        if self.match([TokenType.TRUE]):
            return Literal(value=True)
        if self.match([TokenType.NIL]):
            return Literal(value=None)
        if self.match([TokenType.NUMBER, TokenType.STRING]):
            return Literal(value=self.previous().literal)
        if self.match(
            [
                TokenType.LEFT_PAREN,
            ]
        ):
            expr = self.expression()
            self.consume(token_type=TokenType.RIGHT_PAREN, message="Expect ')' after expression.")
            return Grouping(expression=expr)

    def match(self, tokens_types: list[TokenType]) -> bool:
        for token in tokens_types:
            if self.check(token):
                self.advance()
                return True
        else:
            return False

    def check(self, token_type: TokenType) -> bool:
        if self.is_at_end():
            return False
        return self.peek().token_type == token_type

    def advance(self) -> Token:
        if not self.is_at_end():
            self.current += 1
        return self.previous()

    def is_at_end(self) -> bool:
        return self.peek().token_type == TokenType.EOF

    def peek(self) -> Token:
        return self.tokens[self.current]

    def previous(self) -> Token:
        return self.tokens[self.current - 1]

    def consume(self, token_type: TokenType, message: str) -> Token:
        if self.check(token_type):
            return self.advance()
        else:
            raise PassingError(f"{self.peek()} {message}")
