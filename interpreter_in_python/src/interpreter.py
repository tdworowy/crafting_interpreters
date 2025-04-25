from src.expr import (
    Assign,
    Binary,
    Call,
    Expr,
    Get,
    Grouping,
    Literal,
    Logical,
    Set,
    Super,
    T,
    This,
    Unary,
    Variable,
    Visitor,
)
from src.token_ import TokenType


class Interpreter(Visitor):

    def evaluate(self, expr: Expr):
        return expr.accept(self)

    def visit_assign_expr(self, expr: Assign) -> T:
        raise NotImplementedError

    def visit_binary_expr(self, expr: Binary) -> T:
        left = self.evaluate(expr.left)
        right = self.evaluate(expr.right)
        match expr.operator.token_type:
            case TokenType.MINUS:
                return left - right
            case TokenType.PLUS:
                return left + right
            case TokenType.SLASH:
                return left / right
            case TokenType.STAR:
                return left * right
            case TokenType.GREATER:
                return left > right
            case TokenType.GREATER_EQUAL:
                return left >= right
            case TokenType.LESS:
                return left < right
            case TokenType.LESS_EQUAL:
                return left <= right
            case TokenType.BANG_EQUAL:
                return left != right
            case TokenType.EQUAL_EQUAL:
                return left == right
            case _:
                return None

    def visit_call_expr(self, expr: Call) -> T:
        raise NotImplementedError

    def visit_get_expr(self, expr: Get) -> T:
        raise NotImplementedError

    def visit_grouping_expr(self, expr: Grouping) -> T:
        return self.evaluate(expr=expr.expression)

    def visit_literal_expr(self, expr: Literal) -> T:
        return expr.value

    def visit_logical_expr(self, expr: Logical) -> T:
        raise NotImplementedError

    def visit_set_expr(self, expr: Set) -> T:
        raise NotImplementedError

    def visit_super_expr(self, expr: Super) -> T:
        raise NotImplementedError

    def visit_this_expr(self, expr: This) -> T:
        return "this"

    def visit_unary_expr(self, expr: Unary) -> T:
        right = self.evaluate(expr=expr.right)
        match expr.operator.token_type:

            case TokenType.BANG:
                return not right
            case TokenType.MINUS:
                return -right
            case _:
                return None

    def visit_variable_expr(self, expr: Variable) -> T:
        return expr.name.lexeme
