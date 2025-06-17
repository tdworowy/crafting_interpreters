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
    FunctionExpr,
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
    METHOD = auto()
    INITIALIZER = auto()


class ClassType(Enum):
    NONE = auto()
    CLASS = auto()


class Resolver(VisitorExpr, VisitorStmt):

    def __init__(self, interpreter: Interpreter):
        self.interpreter = interpreter
        self.scopes = []
        self.had_error = False
        self.current_function = FunctionType.NONE
        self.current_class = ClassType.NONE

    def visit_block_stmt(self, stmt: "Block") -> None:
        self.begin_scope()
        self.resolve(to_resolve=stmt.statements)
        self.end_scope()

    def visit_break_stmt(self, stmt: "Break") -> T:
        pass

    def visit_class_stmt(self, stmt: "Class") -> None:
        enclosing_class = self.current_class
        self.current_class = ClassType.CLASS

        self.declare(name=stmt.name)
        self.define(name=stmt.name)
        self.begin_scope()
        self.scopes[-1]["this"] = True
        for method in stmt.class_methods:
            self.begin_scope()
            self.scopes[-1]["this"] = True
            self.resolve_function(
                function_param=method, function_type=FunctionType.METHOD
            )
            self.end_scope()

        for method in stmt.methods:
            if method.name.lexeme == "init":
                declaration = FunctionType.INITIALIZER
            else:
                declaration = FunctionType.METHOD
            self.resolve_function(function_param=method, function_type=declaration)
        self.end_scope()

        self.current_class = enclosing_class

    def visit_expression_stmt(self, stmt: "Expression") -> None:
        self.resolve(to_resolve=stmt.expression)

    def visit_function_stmt(self, stmt: "FunctionStmt") -> None:
        self.declare(name=stmt.name)
        self.define(name=stmt.name)
        self.resolve_function(function_param=stmt, function_type=FunctionType.FUNCTION)

    def visit_function_expr(self, expr: "FunctionExpr"):
        self.resolve_function(function_param=expr, function_type=FunctionType.FUNCTION)

    def resolve_function(
        self, function_param: FunctionStmt | FunctionExpr, function_type: FunctionType
    ):
        match function_param:
            case FunctionExpr():
                function = function_param
            case FunctionStmt():
                function = function_param.function

        enclosing_function = self.current_function
        self.current_function = function_type
        self.begin_scope()
        for param in function.params:
            self.declare(name=param)
            self.define(name=param)
        self.resolve(to_resolve=function.body)
        self.end_scope()
        self.current_function = enclosing_function

    def visit_if_stmt(self, stmt: "If") -> None:
        self.resolve(to_resolve=stmt.condition)
        self.resolve(to_resolve=stmt.then_branch)
        if stmt.else_branch:
            self.resolve(to_resolve=stmt.else_branch)

    def visit_print_stmt(self, stmt: "Print") -> None:
        self.resolve(to_resolve=stmt.expression)

    def visit_return_stmt(self, stmt: "Return") -> None:
        if self.current_function == FunctionType.NONE:
            print("Can't return from top-level code.")
            self.had_error = True
            return
        if stmt.value:
            if self.current_function == FunctionType.INITIALIZER:
                print("Can't return from initializer.")
                self.had_error = True
                return
            self.resolve(to_resolve=stmt.value)

    def visit_var_stmt(self, stmt: "Var") -> None:
        self.declare(name=stmt.name)
        if stmt.initializer is not None:
            self.resolve(to_resolve=stmt.initializer)
        self.define(name=stmt.name)

    def declare(self, name: Token):
        if not self.scopes:
            return
        if name in self.scopes[-1].keys():
            print("Already a variable with this name is this scope.")
            self.had_error = True
            return
        self.scopes[-1][name.lexeme] = False

    def define(self, name: Token):
        if not self.scopes:
            return
        self.scopes[-1][name.lexeme] = True

    def visit_while_stmt(self, stmt: "While") -> None:
        self.resolve(to_resolve=stmt.condition)
        self.resolve(to_resolve=stmt.body)

    def visit_variable_expr(self, expr: "Variable") -> None:
        if self.scopes and self.scopes[-1].get(expr.name.lexeme) is False:
            self.had_error = True
            print(
                f"Can't read local variable '{expr.name.lexeme}' in its own initializer"
            )
            return
        self.resolve_local(expr=expr, name=expr.name)

    def resolve_local(self, expr: Expr, name: Token):
        i = len(self.scopes)
        for scope in reversed(self.scopes):
            if name.lexeme in scope.keys():
                self.interpreter.resolve(expr=expr, depth=len(self.scopes) - i)
            i -= 1

    def visit_this_expr(self, expr: "This") -> None:
        if self.current_class == ClassType.NONE:
            print(f"Can't use 'this' outside of class.")
            self.had_error = True
            return
        self.resolve_local(expr=expr, name=expr.keyword)

    def visit_unary_expr(self, expr: "Unary") -> None:
        self.resolve(to_resolve=expr.right)

    def visit_super_expr(self, expr: "Super") -> T:
        pass

    def visit_set_expr(self, expr: "Set") -> None:
        self.resolve(to_resolve=expr.value)
        self.resolve(to_resolve=expr.object)

    def visit_logical_expr(self, expr: "Logical") -> None:
        self.resolve(to_resolve=expr.left)
        self.resolve(to_resolve=expr.right)

    def visit_literal_expr(self, expr: "Literal") -> None:
        return

    def visit_grouping_expr(self, expr: "Grouping") -> None:
        self.resolve(to_resolve=expr.expression)

    def visit_get_expr(self, expr: "Get") -> None:
        self.resolve(to_resolve=expr.object)

    def visit_call_expr(self, expr: "Call") -> None:
        self.resolve(to_resolve=expr.callee)
        for argument in expr.arguments:
            self.resolve(to_resolve=argument)

    def visit_binary_expr(self, expr: "Binary") -> None:
        self.resolve(to_resolve=expr.left)
        self.resolve(to_resolve=expr.right)

    def visit_assign_expr(self, expr: "Assign") -> None:
        self.resolve(to_resolve=expr.value)
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
