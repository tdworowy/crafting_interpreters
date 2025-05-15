from src.expr import Assign, Binary, Expr, Grouping, Literal, Unary, Variable, Logical
from src.stmt import Block, Expression, Print, Stmt, Var, If, While, Break
from src.token_ import Token, TokenType


class ParseError(Exception):
    def __init__(self, message):
        super().__init__(message)


class Parser:
    def __init__(self, tokens: list[Token]):
        self.tokens = tokens
        self.current = 0
        self.loop_depth = 0
        self.allow_expression = False
        self.found_expression = False
        self.had_error = False

    def get_parsing_error(self, token: Token, message: str) -> ParseError:
        if token.token_type == TokenType.EOF:
            message = f"{token.line} at end [{message}]"
        else:
            message = f"{token.line} at {token.lexeme} [{message}]"
        self.had_error = True
        print(message)
        return ParseError(message)

    def parse(self) -> list[Stmt]:
        statements = []
        while not self.is_at_end():
            statements.append(self.declaration())
        return statements

    def parse_repl(self) -> list[Stmt] | Expr:
        self.allow_expression = True
        statements = []
        while not self.is_at_end():
            statements.append(self.declaration())
            if self.found_expression:
                last: Expr = statements[-1].expression
                return last
            self.allow_expression = False
        return statements

    def declaration(self) -> Stmt:
        try:
            if self.match(tokens_types=[TokenType.VAR]):
                return self.var_declaration()
            else:
                return self.statement()
        except ParseError:
            self.synchronize()

    def var_declaration(self) -> Stmt:
        name = self.consume(
            token_type=TokenType.IDENTIFIER, message="Expect variable name."
        )
        initializer = None
        if self.match(tokens_types=[TokenType.EQUAL]):
            initializer = self.expression()

        self.consume(
            token_type=TokenType.SEMICOLON,
            message="Expect ';' after variable declaration.",
        )
        return Var(name=name, initializer=initializer)

    def statement(self) -> Stmt:
        if self.match(tokens_types=[TokenType.FOR]):
            return self.for_statement()
        if self.match(tokens_types=[TokenType.IF]):
            return self.if_statement()
        if self.match(tokens_types=[TokenType.PRINT]):
            return self.print_statement()
        if self.match(tokens_types=[TokenType.WHILE]):
            return self.while_statement()
        if self.match(tokens_types=[TokenType.BREAK]):
            return self.break_statement()

        if self.match(tokens_types=[TokenType.LEFT_BRACE]):
            return Block(self.block())

        return self.expression_statement()

    def for_statement(self) -> Stmt:
        self.consume(token_type=TokenType.LEFT_PAREN, message="Expect '(' after 'for'.")
        initializer = None
        if self.match(tokens_types=[TokenType.SEMICOLON]):
            initializer = None
        elif self.match(tokens_types=[TokenType.VAR]):
            initializer = self.var_declaration()
        else:
            initializer = self.expression_statement()

        condition = None
        if not self.check(token_type=TokenType.SEMICOLON):
            condition = self.expression()
        self.consume(
            token_type=TokenType.SEMICOLON, message="Expect ';' after loop condition."
        )
        increment = None
        if not self.check(token_type=TokenType.RIGHT_PAREN):
            increment = self.expression()
        self.consume(
            token_type=TokenType.RIGHT_PAREN, message="Expect ')' after for clauses."
        )
        try:
            self.loop_depth += 1
            body = self.statement()
            if increment:
                body = Block([body, Expression(expression=increment)])
            if condition is None:
                condition = Literal(value=True)
            body = While(condition=condition, body=body)
            if initializer:
                body = Block([initializer, body])
            return body
        finally:
            self.loop_depth -= 1

    def if_statement(self) -> Stmt:
        self.consume(
            token_type=TokenType.LEFT_PAREN, message="Expected '(' after 'if'."
        )
        condition = self.expression()
        self.consume(
            token_type=TokenType.RIGHT_PAREN,
            message="Expected ')' after 'if' condition.",
        )
        then_branch = self.statement()
        else_branch = None
        if self.match(tokens_types=[TokenType.ELSE]):
            else_branch = self.statement()

        return If(condition=condition, then_branch=then_branch, else_branch=else_branch)

    def print_statement(self) -> Stmt:
        value = self.expression()
        self.consume(token_type=TokenType.SEMICOLON, message="Expect ';' after value.")
        return Print(expression=value)

    def while_statement(self) -> Stmt:
        self.consume(
            token_type=TokenType.LEFT_PAREN, message="Expect '(' after 'while'."
        )
        condition = self.expression()
        self.consume(
            token_type=TokenType.RIGHT_PAREN, message="Expect ')' after 'condition'."
        )
        try:
            self.loop_depth += 1
            body = self.statement()
            return While(condition=condition, body=body)
        finally:
            self.loop_depth -= 1

    def break_statement(self) -> Stmt:
        if self.loop_depth == 0:
            self.get_parsing_error(
                token=self.previous(), message="Must be inside a loop to use 'break'."
            )
        self.consume(
            token_type=TokenType.SEMICOLON, message="Expect ';' after 'break'."
        )
        return Break()

    def block(self) -> list[Stmt]:
        statements = []
        while not self.check(token_type=TokenType.RIGHT_BRACE) and not self.is_at_end():
            statements.append(self.declaration())
        self.consume(
            token_type=TokenType.RIGHT_BRACE, message="Expected '}' after block."
        )
        return statements

    def expression_statement(self) -> Stmt:
        expr = self.expression()
        if self.allow_expression and self.is_at_end():
            self.found_expression = True
        else:
            self.consume(
                token_type=TokenType.SEMICOLON, message="Expect ';' after value."
            )
        return Expression(expression=expr)

    def expression(self) -> Expr:
        return self.assignment()

    def assignment(self) -> Expr:
        expr = self.logical_or()
        if self.match([TokenType.EQUAL]):
            equals = self.previous()
            value = self.assignment()
            if isinstance(expr, Variable):
                name = expr.name
                return Assign(name=name, value=value)
            self.get_parsing_error(
                token=equals,
                message=f"Invalid assignment target.",
            )
        else:
            return expr

    def logical_or(self) -> Expr:
        expr = self.logical_and()
        while self.match(tokens_types=[TokenType.OR]):
            operator = self.previous()
            right = self.logical_and()
            expr = Logical(left=expr, operator=operator, right=right)
        return expr

    def logical_and(self) -> Expr:
        expr = self.equality()
        while self.match(tokens_types=[TokenType.AND]):
            operator = self.previous()
            right = self.logical_and()
            expr = Logical(left=expr, operator=operator, right=right)
        return expr

    def equality(self) -> Expr:
        expr = self.comparison()
        while self.match(tokens_types=[TokenType.BANG_EQUAL, TokenType.EQUAL_EQUAL]):
            operator = self.previous()
            right = self.comparison()
            expr = Binary(left=expr, operator=operator, right=right)
        return expr

    def comparison(self) -> Expr:
        expr = self.term()
        while self.match(
            tokens_types=[
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
        while self.match(tokens_types=[TokenType.MINUS, TokenType.PLUS]):
            operator = self.previous()
            right = self.factor()
            expr = Binary(left=expr, operator=operator, right=right)
        return expr

    def factor(self) -> Expr:
        expr = self.unary()
        while self.match(tokens_types=[TokenType.SLASH, TokenType.STAR]):
            operator = self.previous()
            right = self.unary()
            expr = Binary(left=expr, operator=operator, right=right)
        return expr

    def unary(self) -> Expr:
        if self.match(tokens_types=[TokenType.BANG, TokenType.MINUS]):
            operator = self.previous()
            right = self.unary()
            return Unary(operator=operator, right=right)
        elif self.match(tokens_types=[TokenType.PLUS, TokenType.STAR, TokenType.SLASH]):
            raise self.get_parsing_error(
                token=self.previous(),
                message=f"Binary operator without left-hand operand",
            )
        return self.primary()

    def primary(self) -> Expr:
        if self.match(tokens_types=[TokenType.FALSE]):
            return Literal(value=False)
        if self.match(tokens_types=[TokenType.TRUE]):
            return Literal(value=True)
        if self.match(tokens_types=[TokenType.NIL]):
            return Literal(value=None)
        if self.match(tokens_types=[TokenType.NUMBER, TokenType.STRING]):
            return Literal(value=self.previous().literal)
        if self.match(tokens_types=[TokenType.IDENTIFIER]):
            return Variable(name=self.previous())
        if self.match(
            tokens_types=[
                TokenType.LEFT_PAREN,
            ]
        ):
            expr = self.expression()
            self.consume(
                token_type=TokenType.RIGHT_PAREN, message="Expect ')' after expression."
            )
            return Grouping(expression=expr)

        raise self.get_parsing_error(
            token=self.peek(),
            message=f"Expected expression.",
        )

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
            raise self.get_parsing_error(token=self.peek(), message=message)

    def synchronize(self):
        self.advance()
        while not self.is_at_end():
            if self.previous().token_type == TokenType.SEMICOLON:
                return
            if self.peek().token_type in [
                TokenType.CLASS,
                TokenType.FUN,
                TokenType.VAR,
                TokenType.FOR,
                TokenType.IF,
                TokenType.WHILE,
                TokenType.PRINT,
                TokenType.RETURN,
            ]:
                return
            self.advance()
