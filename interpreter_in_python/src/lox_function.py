from typing import Optional

from src.environment import Environment
from src.expr import FunctionExpr
from src.lox_instance import LoxInstance
from src.return_exception import ReturnException
from src.lox_callable import LoxCallable
from src.stmt import FunctionStmt
from src.token_ import Token


class LoxFunction(LoxCallable):
    def __init__(
        self,
        name: Optional[str],
        declaration: FunctionExpr | FunctionStmt,
        closure: Environment,
    ):
        self.name = name
        self.closure = closure
        match declaration:
            case FunctionExpr():
                self.declaration = declaration
            case FunctionStmt():
                self.declaration = declaration.function

    def call(self, interpreter: "Interpreter", arguments: list) -> Token:
        environment = Environment(enclosing=self.closure)
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

    def bind(self, instance: LoxInstance):
        environment = Environment(enclosing=self.closure)
        environment.define(name="this", value=instance)
        return LoxFunction(
            name="this", declaration=self.declaration, closure=environment
        )

    def __str__(self):
        if not self.name:
            return "<fn>"
        return f"<fn {self.name}>"

    def __repr__(self):
        if not self.name:
            return "<fn>"
        return f"<fn {self.name}>"
