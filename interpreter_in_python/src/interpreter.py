from multimethod import multimethod

from src.environment import Environment
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
from src.run_time_exception import RuneTimeException
from src.stmt import (
    Block,
    Class,
    Expression,
    Function,
    If,
    Print,
    Return,
    Stmt,
    Var,
    VisitorStmt,
    While,
)
from src.token_ import Token, TokenType


@multimethod
def check_number_operator(operator: Token, operand: str | int | float):
    match operand:
        case int() | float():
            return
        case _:
            raise RuneTimeException(
                token=operator, message=f"Operand {operand} must be number."
            )


@multimethod
def check_number_operator(
    operator: Token, left: str | int | float, right: str | int | float
):
    match (left, right):
        case (int() | float(), int() | float()):
            return
        case _:
            raise RuneTimeException(
                token=operator, message=f"Operands [{left},{right}] must be numbers."
            )


def check_same_type(operator: Token, left: str | int | float, right: str | int | float):
    if type(left) is type(right):
        return
    else:
        raise RuneTimeException(
            token=operator, message=f"Operands [{left},{right}] need to had same type."
        )


class Interpreter(VisitorExpr, VisitorStmt):

    def __init__(self):
        self.had_error = False
        self.environment = Environment({})

    def run_time_error(self, error: RuneTimeException):
        self.had_error = True
        print(f"{error.message}\n[line {error.token.line} ]")

    def interpret(self, statements: list[Stmt]):
        try:
            for stmt in statements:
                self.execute(stmt)

        except RuneTimeException as etx:
            self.run_time_error(etx)

    def evaluate(self, expr: Expr) -> T:
        return expr.accept(self)

    def execute(self, stmt: Stmt) -> T:
        return stmt.accept(self)

    def visit_class_stmt(self, stmt: "Class") -> T:
        pass

    def visit_expression_stmt(self, stmt: "Expression") -> None:
        self.evaluate(stmt.expression)

    def visit_function_stmt(self, stmt: "Function") -> T:
        pass

    def visit_if_stmt(self, stmt: "If") -> T:
        pass

    def visit_print_stmt(self, stmt: "Print") -> None:
        value = self.evaluate(stmt.expression)
        print(value)

    def visit_return_stmt(self, stmt: "Return") -> T:
        pass

    def visit_var_stmt(self, stmt: "Var") -> None:
        value = None
        if stmt.initializer is not None:
            value = self.evaluate(expr=stmt.initializer)
        self.environment.define(name=stmt.name.lexeme, value=value)

    def visit_while_stmt(self, stmt: "While") -> T:
        pass

    def visit_block_stmt(self, stmt: "Block") -> T:
        pass

    def visit_assign_expr(self, expr: Assign) -> T:
        raise NotImplementedError

    def visit_binary_expr(self, expr: Binary) -> T:
        left = self.evaluate(expr.left)
        right = self.evaluate(expr.right)
        match expr.operator.token_type:
            case TokenType.MINUS:
                check_number_operator(expr.operator, left, right)
                return left - right
            case TokenType.PLUS:
                check_same_type(expr.operator, left, right)
                return left + right
            case TokenType.SLASH:
                check_number_operator(expr.operator, left, right)
                if right == 0:
                    raise RuneTimeException(
                        token=expr.operator,
                        message=f"Division be zero [{left}/{right}].",
                    )
                return left / right
            case TokenType.STAR:
                check_number_operator(expr.operator, left, right)
                return left * right
            case TokenType.GREATER:
                check_number_operator(expr.operator, left, right)
                return left > right
            case TokenType.GREATER_EQUAL:
                check_number_operator(expr.operator, left, right)
                return left >= right
            case TokenType.LESS:
                check_number_operator(expr.operator, left, right)
                return left < right
            case TokenType.LESS_EQUAL:
                check_number_operator(expr.operator, left, right)
                return left <= right
            case TokenType.BANG_EQUAL:
                check_same_type(expr.operator, left, right)
                return left != right
            case TokenType.EQUAL_EQUAL:
                check_same_type(expr.operator, left, right)
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
                check_number_operator(expr.operator, right)
                return -right
            case _:
                return None

    def visit_variable_expr(self, expr: Variable) -> T:
        return self.environment.get(name=expr.name)
