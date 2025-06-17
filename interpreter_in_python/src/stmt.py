from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import TypeVar

from src.expr import Expr, Variable, FunctionExpr
from src.token_ import Token

T = TypeVar("T")


class VisitorStmt(ABC):
    @abstractmethod
    def visit_block_stmt(self, stmt: "Block") -> T:
        pass

    @abstractmethod
    def visit_break_stmt(self, stmt: "Break") -> T:
        pass

    @abstractmethod
    def visit_class_stmt(self, stmt: "Class") -> T:
        pass

    @abstractmethod
    def visit_expression_stmt(self, stmt: "Expression") -> T:
        pass

    @abstractmethod
    def visit_function_stmt(self, stmt: "FunctionStmt") -> T:
        pass

    @abstractmethod
    def visit_if_stmt(self, stmt: "If") -> T:
        pass

    @abstractmethod
    def visit_print_stmt(self, stmt: "Print") -> T:
        pass

    @abstractmethod
    def visit_return_stmt(self, stmt: "Return") -> T:
        pass

    @abstractmethod
    def visit_var_stmt(self, stmt: "Var") -> T:
        pass

    @abstractmethod
    def visit_while_stmt(self, stmt: "While") -> T:
        pass


class Stmt(ABC):

    @abstractmethod
    def accept(self, visitor: VisitorStmt) -> T:
        pass


@dataclass
class Block(Stmt):
    statements: list[Stmt]

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_block_stmt(self)


@dataclass
class Break(Stmt):

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_break_stmt(self)


@dataclass
class Expression(Stmt):
    expression: Expr

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_expression_stmt(self)


@dataclass
class FunctionStmt(Stmt):
    name: Token
    function: FunctionExpr

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_function_stmt(self)


@dataclass
class Class(Stmt):
    name: Token
    supper_class: Variable | None
    methods: list[FunctionStmt]
    class_methods: list[FunctionStmt]

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_class_stmt(self)


@dataclass
class If(Stmt):
    condition: Expr
    then_branch: Stmt
    else_branch: Stmt | None

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_if_stmt(self)


@dataclass
class Print(Stmt):
    expression: Expr

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_print_stmt(self)


@dataclass
class Return(Stmt):
    keyword: Token
    value: Expr

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_return_stmt(self)


@dataclass
class Var(Stmt):
    name: Token
    initializer: Expr

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_var_stmt(self)


@dataclass
class While(Stmt):
    condition: Expr
    body: Stmt

    def accept(self, visitor: VisitorStmt) -> T:
        return visitor.visit_while_stmt(self)
