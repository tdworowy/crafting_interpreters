from multimethod import multimethod

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


class Resolver(VisitorExpr, VisitorStmt):

    def __init__(self, interpreter: Interpreter):
        self.interpreter = interpreter
        self.scopes = []

    def visit_block_stmt(self, stmt: "Block") -> None:
        self.begin_scope()
        self.resolve(stmt.statements)
        self.end_scope()

    def visit_break_stmt(self, stmt: "Break") -> T:
        pass

    def visit_class_stmt(self, stmt: "Class") -> T:
        pass

    def visit_expression_stmt(self, stmt: "Expression") -> T:
        pass

    def visit_function_stmt(self, stmt: "FunctionStmt") -> T:
        pass

    def visit_if_stmt(self, stmt: "If") -> T:
        pass

    def visit_print_stmt(self, stmt: "Print") -> T:
        pass

    def visit_return_stmt(self, stmt: "Return") -> T:
        pass

    def visit_var_stmt(self, stmt: "Var") -> T:
        self.declare(name=stmt.name)
        if stmt.initializer is not None:
            self.resolve(stmt.initializer)
        self.define(name=stmt.name)

    def declare(self, name: Token):
        if len(self.scopes) == 0:
            return
        self.scopes[-1][name.lexeme] = False

    def define(self, name: Token):
        if len(self.scopes) == 0:
            return
        self.scopes[-1][name.lexeme] = False

    def visit_while_stmt(self, stmt: "While") -> T:
        pass

    def visit_variable_expr(self, expr: "Variable") -> T:
        pass

    def visit_this_expr(self, expr: "This") -> T:
        pass

    def visit_unary_expr(self, expr: "Unary") -> T:
        pass

    def visit_super_expr(self, expr: "Super") -> T:
        pass

    def visit_set_expr(self, expr: "Set") -> T:
        pass

    def visit_logical_expr(self, expr: "Logical") -> T:
        pass

    def visit_literal_expr(self, expr: "Literal") -> T:
        pass

    def visit_grouping_expr(self, expr: "Grouping") -> T:
        pass

    def visit_get_expr(self, expr: "Get") -> T:
        pass

    def visit_call_expr(self, expr: "Call") -> T:
        pass

    def visit_binary_expr(self, expr: "Binary") -> T:
        pass

    def visit_assign_expr(self, expr: "Assign") -> T:
        pass

    @multimethod
    def resolve(self, statements: list[Stmt]):
        for statement in statements:
            self.resolve(statement)

    @multimethod
    def resolve(self, stmt: Stmt):
        stmt.accept(self)

    @multimethod
    def resolve(self, expr: Expr):
        expr.accept(self)

    def begin_scope(self):
        self.scopes.append({})

    def end_scope(self):
        self.scopes.pop()
