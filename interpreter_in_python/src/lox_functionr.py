from typing import Optional

from src.environment import Environment
from src.expr import FunctionExpr
from src.return_exception import ReturnException
from src.lox_callable import LoxCallable
from src.token_ import Token


class LoxFunction(LoxCallable):
    def __init__(
        self, name: Optional[str], declaration: FunctionExpr, closure: Environment
    ):
        self.name = name
        self.declaration = declaration
        self.closure = closure

    def call(self, interpreter: "Interpreter", arguments: list) -> Token:
        environment = Environment(values={}, enclosing=self.closure)
        for param, argument in zip(self.declaration.params, arguments):
            environment.define(name=param.lexeme, value=argument)
        try:
            interpreter.execute_block(
                statements=self.declaration.body, environment=environment
            )
        except ReturnException as return_value:
            return return_value.token

    def arity(self) -> int:
        return len(self.declaration.params)

    def __str__(self):
        if not self.name:
            return "<fn>"
        return f"<fn {self.name}>"

    def __repr__(self):
        if not self.name:
            return "<fn>"
        return f"<fn {self.name}>"
