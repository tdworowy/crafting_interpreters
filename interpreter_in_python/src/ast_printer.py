from typing import Any

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
    VisitorExpr,
)
from src.token_ import Token


class AstPrinter(VisitorExpr):

    def print_ast(self, expr: Expr):
        return expr.accept(self)

    def visit_assign_expr(self, expr: Assign) -> T:
        return self.parenthesize2("=", expr.name.lexeme, expr.value)

    def visit_binary_expr(self, expr: Binary) -> T:
        return self.parenthesize1(expr.operator.lexeme, expr.left, expr.right)

    def visit_call_expr(self, expr: Call) -> T:
        return self.parenthesize2("call", expr.callee, expr.arguments)

    def visit_get_expr(self, expr: Get) -> T:
        return self.parenthesize2(".", expr.object, expr.name.lexeme)

    def visit_grouping_expr(self, expr: Grouping) -> T:
        return self.parenthesize1("group", expr.expression)

    def visit_literal_expr(self, expr: Literal) -> T:
        if expr.value is None:
            return "nil"
        else:
            return expr.value

    def visit_logical_expr(self, expr: Logical) -> T:
        return self.parenthesize1(expr.operator.lexeme, expr.left, expr.right)

    def visit_set_expr(self, expr: Set) -> T:
        return self.parenthesize2("=", expr.name.lexeme, expr.value)

    def visit_super_expr(self, expr: Super) -> T:
        return self.parenthesize2("super", expr.method)

    def visit_this_expr(self, expr: This) -> T:
        return "this"

    def visit_unary_expr(self, expr: Unary) -> T:
        return self.parenthesize1(expr.operator.lexeme, expr.right)

    def visit_variable_expr(self, expr: Variable) -> T:
        return expr.name.lexeme

    def parenthesize1(self, name: str, *exprs) -> str:
        result = f"({name}"
        for expr in exprs:
            result += " "
            result += str(expr.accept(self))
        result += ")"
        return result

    def parenthesize2(self, name: str, *parts) -> str:
        result = f"({name}"
        result += self.transform(parts)
        result += ")"
        return result

    def transform(self, parts: tuple[Any, ...]):
        result = ""
        for part in parts:
            result += " "
            match part:
                case Expr():
                    result += str(part.accept(self))
                case Token():
                    result += part.lexeme
                case list():
                    result += self.transform(part)
                case _:
                    result += part
        return result
