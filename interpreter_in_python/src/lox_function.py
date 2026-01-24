from src.environment import Environment
from src.expr import FunctionExpr
from src.lox_callable import LoxCallable
from src.lox_instance import LoxInstance
from src.return_exception import ReturnException
from src.stmt import FunctionStmt
from src.token_ import Token


class LoxFunction(LoxCallable):
    def __init__(
        self,
        name: str | None,
        declaration: FunctionExpr | FunctionStmt,
        closure: Environment,
        is_initializer: bool = False,
    ):
        self.name = name
        self.closure = closure
        self.is_initializer = is_initializer
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
            if self.is_initializer:
                return self.closure.get_at(distance=0, name="this")
            else:
                return return_value.token

        if self.is_initializer:
            return self.closure.get_at(distance=0, name="this")

    def arity(self) -> int:
        return len(self.declaration.params)

    def bind(self, instance: LoxInstance):
        environment = Environment(enclosing=self.closure)
        environment.define(name="this", value=instance)
        return LoxFunction(
            name="this",
            declaration=self.declaration,
            closure=environment,
            is_initializer=self.is_initializer,
        )

    def __str__(self):
        if not self.name:
            return "<fn>"
        return f"<fn {self.name}>"

    def __repr__(self):
        if not self.name:
            return "<fn>"
        return f"<fn {self.name}>"
