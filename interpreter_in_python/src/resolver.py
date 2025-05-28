from enum import Enum, auto

from src.expr import (
    VisitorExpr,
    T,
    Variable,
    This,
    Unary,
    Super,
    Set,
    Logical,
    Literal,
    Grouping,
    Get,
    Call,
    Binary,
    Assign,
    Expr,
)
from src.interpreter import Interpreter
from src.stmt import (
    VisitorStmt,
    Block,
    Break,
    Class,
    Expression,
    FunctionStmt,
    If,
    Print,
    Return,
    Var,
    While,
    Stmt,
)
from src.token_ import Token


class FunctionType(Enum):
    NONE = auto()
    FUNCTION = auto()


class Resolver(VisitorExpr, VisitorStmt):

    def __init__(self, interpreter: Interpreter):
        self.interpreter = interpreter
        self.scopes = []
        self.had_error = False
        self.current_function = FunctionType.NONE

    def visit_block_stmt(self, stmt: "Block") -> None:
        self.begin_scope()
        self.resolve(stmt.statements)
        self.end_scope()

    def visit_break_stmt(self, stmt: "Break") -> T:
        pass

    def visit_class_stmt(self, stmt: "Class") -> T:
        pass

    def visit_expression_stmt(self, stmt: "Expression") -> None:
        self.resolve(stmt.expression)

    def visit_function_stmt(self, stmt: "FunctionStmt") -> None:
        self.declare(name=stmt.name)
        self.define(name=stmt.name)
        self.resolve_function(function_stmt=stmt, function_type=FunctionType.FUNCTION)

    def resolve_function(
        self, function_stmt: FunctionStmt, function_type: FunctionType
    ):
        enclosing_function = self.current_function
        self.current_function = function_type
        self.begin_scope()
        for param in function_stmt.function.params:
            self.declare(name=param)
            self.define(name=param)
        self.resolve(function_stmt.function.body)
        self.end_scope()
        self.current_function = enclosing_function

    def visit_if_stmt(self, stmt: "If") -> None:
        self.resolve(stmt.condition)
        self.resolve(stmt.then_branch)
        if stmt.else_branch:
            self.resolve(stmt.else_branch)

    def visit_print_stmt(self, stmt: "Print") -> None:
        self.resolve(stmt.expression)

    def visit_return_stmt(self, stmt: "Return") -> None:
        if self.current_function == FunctionType.NONE:
            print("Can't return from top-level code.")
            self.had_error = True
            return
        if stmt.value:
            self.resolve(stmt.value)

    def visit_var_stmt(self, stmt: "Var") -> None:
        self.declare(name=stmt.name)
        if stmt.initializer is not None:
            self.resolve(stmt.initializer)
        self.define(name=stmt.name)

    def declare(self, name: Token):
        if len(self.scopes) == 0:
            return
        if name in self.scopes[-1].keys():
            print("Already a variable with this name is this scope.")
            self.had_error = True
            return
        self.scopes[-1][name.lexeme] = False

    def define(self, name: Token):
        if len(self.scopes) == 0:
            return
        self.scopes[-1][name.lexeme] = True

    def visit_while_stmt(self, stmt: "While") -> None:
        self.resolve(stmt.condition)
        self.resolve(stmt.body)

    def visit_variable_expr(self, expr: "Variable") -> None:
        if (
            len(self.scopes) > 0
            and self.scopes[-1].get(expr.name.lexeme, None) is False
        ):
            self.had_error = True
            print("Can't read local variable in its own initializer")
            return
        self.resolve_local(expr=expr, name=expr.name)

    def resolve_local(self, expr: Expr, name: Token):
        i = len(self.scopes)
        for scope in reversed(self.scopes):
            if name.lexeme in scope.keys():
                self.interpreter.resolve(expr=expr, depth=len(self.scopes) - 1 - i)
            i -= 1

    def visit_this_expr(self, expr: "This") -> T:
        pass

    def visit_unary_expr(self, expr: "Unary") -> None:
        self.resolve(expr.right)

    def visit_super_expr(self, expr: "Super") -> T:
        pass

    def visit_set_expr(self, expr: "Set") -> T:
        pass

    def visit_logical_expr(self, expr: "Logical") -> None:
        self.resolve(expr.left)
        self.resolve(expr.right)

    def visit_literal_expr(self, expr: "Literal") -> None:
        return

    def visit_grouping_expr(self, expr: "Grouping") -> None:
        self.resolve(expr.expression)

    def visit_get_expr(self, expr: "Get") -> T:
        pass

    def visit_call_expr(self, expr: "Call") -> None:
        self.resolve(expr.callee)
        for argument in expr.arguments:
            self.resolve(argument)

    def visit_binary_expr(self, expr: "Binary") -> None:
        self.resolve(expr.left)
        self.resolve(expr.right)

    def visit_assign_expr(self, expr: "Assign") -> None:
        self.resolve(expr.value)
        self.resolve_local(expr=expr, name=expr.name)

    def resolve(self, to_resolve: list[Stmt] | Stmt | Expr):
        match to_resolve:
            case list():
                for statement in to_resolve:
                    self.resolve(statement)
            case Stmt() | Expr():
                to_resolve.accept(self)

    def begin_scope(self):
        self.scopes.append({})

    def end_scope(self):
        self.scopes.pop()
