import time
from enum import Enum, auto

from multimethod import multimethod

from src.environment import Environment
from src.expr import (
    Assign,
    Binary,
    Call,
    Expr,
    FunctionExpr,
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
from src.lox_callable import LoxCallable
from src.lox_class import LoxClass
from src.lox_function import LoxFunction
from src.lox_instance import LoxInstance
from src.return_exception import ReturnException
from src.run_time_exception import RunTimeException
from src.stmt import (
    Block,
    Break,
    Class,
    Expression,
    FunctionStmt,
    If,
    Print,
    Return,
    Stmt,
    Var,
    VisitorStmt,
    While,
)
from src.token_ import Token, TokenType


class VariableType(Enum):
    INITIALIZED = auto()
    UNINITIALIZED = auto()


class BreakException(RuntimeError):
    pass


@multimethod
def check_number_operator(operator: Token, operand: str | int | float):
    match operand:
        case int() | float():
            return
        case _:
            raise RunTimeException(
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
            raise RunTimeException(
                token=operator, message=f"Operands [{left},{right}] must be numbers."
            )


def check_same_type(operator: Token, left: str | int | float, right: str | int | float):
    if type(left) is type(right):
        return
    else:
        raise RunTimeException(
            token=operator, message=f"Operands [{left},{right}] need to had same type."
        )


class Clock(LoxCallable):
    def arity(self) -> int:
        return 0

    def call(self, interpreter: "Interpreter", arguments: list):
        return time.time()

    def __str__(self):
        return "<nativ fm>"

    def __repr__(self):
        return "<nativ fm>"


class Interpreter(VisitorExpr, VisitorStmt):

    def __init__(self):
        self.had_error = False
        self.globals = Environment()
        self.locals = {}
        self.environment = self.globals

        self.globals.define("clock", Clock())

    def run_time_error(self, error: RunTimeException):
        self.had_error = True
        print(f"{error.message}\n[line {error.token.line} ]")

    def interpret_statements(self, statements: list[Stmt]):
        try:
            for stmt in statements:
                self.execute(stmt=stmt)

        except RunTimeException as etx:
            self.run_time_error(etx)

    def interpret_expression(self, expression: Expr) -> str:
        try:
            return self.evaluate(expression)

        except RunTimeException as etx:
            self.run_time_error(etx)

    def interpret(self, to_interpret: Expr | list[Stmt]) -> str | None:
        match to_interpret:
            case list():
                self.interpret_statements(statements=to_interpret)
            case Expr():
                return self.interpret_expression(expression=to_interpret)

    def evaluate(self, expr: Expr) -> T:
        return expr.accept(self)

    def execute(self, stmt: Stmt) -> T:
        return stmt.accept(self)

    def visit_class_stmt(self, stmt: "Class") -> None:
        super_class = None

        if stmt.super_class:
            super_class = self.evaluate(expr=stmt.super_class)
            if not isinstance(super_class, LoxClass):
                raise RunTimeException(
                    token=stmt.super_class.name, message="Superclass must be a class"
                )

        self.environment.define(name=stmt.name.lexeme, value=None)

        if stmt.super_class:
            self.environment = Environment(enclosing=self.environment)
            self.environment.define(name="super", value=super_class)

        class_methods = {}
        for method in stmt.class_methods:
            function = LoxFunction(
                name=method.name.lexeme,
                declaration=method,
                closure=self.environment,
                is_initializer=False,
            )
            class_methods[method.name.lexeme] = function

        meta_class = LoxClass(
            meta_class=None, name=f"{stmt.name.lexeme}_metaclass", methods=class_methods
        )

        methods = {}
        for method in stmt.methods:
            function = LoxFunction(
                name=method.name.lexeme,
                declaration=method,
                closure=self.environment,
                is_initializer=method.name.lexeme == "init",
            )
            methods[method.name.lexeme] = function
        klass = LoxClass(
            meta_class=meta_class,
            super_class=super_class,
            name=stmt.name.lexeme,
            methods=methods,
        )
        if stmt.super_class:
            self.environment = self.environment.enclosing

        self.environment.assign(name=stmt.name, value=klass)

    def visit_expression_stmt(self, stmt: "Expression") -> None:
        self.evaluate(stmt.expression)

    def visit_function_stmt(self, stmt: "FunctionStmt") -> None:
        fn_name = stmt.name.lexeme
        function = LoxFunction(
            name=fn_name, declaration=stmt.function, closure=self.environment
        )
        self.environment.define(name=fn_name, value=function)

    def visit_function_expr(self, expr: "FunctionExpr") -> LoxFunction:
        return LoxFunction(name=None, declaration=expr, closure=self.environment)

    def visit_if_stmt(self, stmt: "If") -> None:
        if self.evaluate(expr=stmt.condition):
            self.execute(stmt=stmt.then_branch)
        elif stmt.else_branch:
            self.execute(stmt=stmt.else_branch)

    def visit_print_stmt(self, stmt: "Print") -> None:
        value = self.evaluate(stmt.expression)
        print(value)

    def visit_return_stmt(self, stmt: "Return") -> None:
        value = None
        if stmt.value is not None:
            value = self.evaluate(expr=stmt.value)
        raise ReturnException(token=value, message=f"return: {value}")

    def visit_var_stmt(self, stmt: "Var") -> None:
        value = VariableType.UNINITIALIZED
        if stmt.initializer is not None:
            value = self.evaluate(expr=stmt.initializer)
        self.environment.define(name=stmt.name.lexeme, value=value)

    def visit_while_stmt(self, stmt: "While") -> None:
        try:
            while self.evaluate(expr=stmt.condition):
                self.execute(stmt=stmt.body)
        except BreakException:
            pass

    def visit_block_stmt(self, stmt: "Block") -> None:
        self.execute_block(
            statements=stmt.statements,
            environment=Environment(enclosing=self.environment),
        )

    def visit_break_stmt(self, stmt: "Break") -> None:
        raise BreakException

    def visit_assign_expr(self, expr: Assign) -> None:
        value = self.evaluate(expr=expr.value)
        distance = self.locals.get(id(expr), None)
        if distance is not None:
            self.environment.assign_at(distance=distance, name=expr.name, value=value)
        else:
            self.globals.assign(name=expr.name, value=value)
        return value

    def visit_binary_expr(self, expr: Binary) -> T:
        left = self.evaluate(expr=expr.left)
        right = self.evaluate(expr=expr.right)
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
                    raise RunTimeException(
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
        callee = self.evaluate(expr=expr.callee)
        if not isinstance(callee, LoxCallable):
            raise RunTimeException(
                token=expr.paren, message="Can only call functions and classes."
            )
        if len(expr.arguments) != callee.arity():
            raise RunTimeException(
                token=expr.paren,
                message=f"Expected {callee.arity} arguments but got {len(expr.arguments)}.",
            )
        arguments = [self.evaluate(argument) for argument in expr.arguments]
        return callee.call(self, arguments)

    def visit_get_expr(self, expr: Get) -> T:
        object = self.evaluate(expr=expr.object)
        if isinstance(object, LoxInstance):
            return object.get(name=expr.name)
        else:
            raise RunTimeException(
                token=expr.name, message="Only instances have properties."
            )

    def visit_grouping_expr(self, expr: Grouping) -> T:
        return self.evaluate(expr=expr.expression)

    def visit_literal_expr(self, expr: Literal) -> T:
        return expr.value

    def visit_logical_expr(self, expr: Logical) -> T:
        left = self.evaluate(expr=expr.left)
        if expr.operator.token_type == TokenType.OR:
            if left:
                return left
        elif not left:
            return left
        return self.evaluate(expr=expr.right)

    def visit_set_expr(self, expr: Set) -> T:
        object = self.evaluate(expr=expr.object)
        if not isinstance(object, LoxInstance):
            raise RunTimeException(
                token=expr.name, message="Only instance have fields."
            )
        value = self.evaluate(expr=expr.value)
        object.set(name=expr.name, value=value)
        return value

    def visit_super_expr(self, expr: Super) -> T:
        distance = self.locals.get(id(expr), None)
        if distance is not None:
            super_class = self.environment.get_at(distance=distance, name="super")
            object = self.environment.get_at(distance=distance - 1, name="this")
            method = super_class.find_method(name=expr.method.lexeme)
            if not method:
                raise RunTimeException(
                    token=expr.method,
                    message=f"Undefined property {expr.method.lexeme}.",
                )
            return method.bind(instance=object)

    def visit_this_expr(self, expr: This) -> T:
        return self.look_up_variable(name=expr.keyword, expr=expr)

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
        return self.look_up_variable(name=expr.name, expr=expr)

    def look_up_variable(self, name: Token, expr: Expr):
        distance = self.locals.get(id(expr), None)
        if distance is not None:
            return self.environment.get_at(distance=distance, name=name.lexeme)
        else:
            return self.globals.get(name=name)

    def execute_block(self, statements: list[Stmt], environment: Environment):
        previous = self.environment
        try:
            self.environment = environment
            for statement in statements:
                self.execute(stmt=statement)
        finally:
            self.environment = previous

    def resolve(self, expr: Expr, depth: int):
        self.locals[id(expr)] = depth
