import abc
from abc import ABC
from dataclasses import dataclass
from typing import TypeVar

from src.token_ import Token

T = TypeVar("T")


class Visitor(ABC):
    @abc.abstractmethod
    def visit_assign_expr(self, expr: "Assign") -> T:
        pass

    @abc.abstractmethod
    def visit_binary_expr(self, expr: "Binary") -> T:
        pass

    @abc.abstractmethod
    def visit_call_expr(self, expr: "Call") -> T:
        pass

    @abc.abstractmethod
    def visit_get_expr(self, expr: "Get") -> T:
        pass

    @abc.abstractmethod
    def visit_grouping_expr(self, expr: "Grouping") -> T:
        pass

    @abc.abstractmethod
    def visit_literal_expr(self, expr: "Literal") -> T:
        pass

    @abc.abstractmethod
    def visit_logical_expr(self, expr: "Logical") -> T:
        pass

    @abc.abstractmethod
    def visit_set_expr(self, expr: "Set") -> T:
        pass

    @abc.abstractmethod
    def visit_super_expr(self, expr: "Super") -> T:
        pass

    @abc.abstractmethod
    def visit_this_expr(self, expr: "This") -> T:
        pass

    @abc.abstractmethod
    def visit_unary_expr(self, expr: "Unary") -> T:
        pass

    @abc.abstractmethod
    def visit_variable_expr(self, expr: "Variable") -> T:
        pass


class Expr(ABC):

    @abc.abstractmethod
    def accept(self, visitor: Visitor) -> T:
        pass


@dataclass
class Assign(Expr):
    name: Token
    value: Expr

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_assign_expr(self)


@dataclass
class Binary(Expr):
    left: Expr
    operator: Token
    right: Expr

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_binary_expr(self)


@dataclass
class Call(Expr):
    callee: Expr
    paren: Token
    arguments: list[Expr]

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_call_expr(self)


@dataclass
class Get(Expr):
    object: Expr
    name: Token

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_get_expr(self)


@dataclass
class Grouping(Expr):
    expression: Expr

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_grouping_expr(self)


@dataclass
class Literal(Expr):
    value: str | int | float

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_literal_expr(self)


@dataclass
class Logical(Expr):
    left: Expr
    operator: Token
    right: Expr

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_logical_expr(self)


@dataclass
class Set(Expr):
    object: Expr
    name: Token
    value: Expr

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_set_expr(self)


@dataclass
class Super(Expr):
    keyword: Token
    method: Token

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_super_expr(self)


@dataclass
class This(Expr):
    keyword: Token

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_this_expr(self)


@dataclass
class Unary(Expr):
    operator: Token
    right: Expr

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_unary_expr(self)


@dataclass
class Variable(Expr):
    name: Token

    def accept(self, visitor: Visitor) -> T:
        return visitor.visit_variable_expr(self)
