from typing import Any, Optional

from src.run_time_exception import RunTimeException
from src.token_ import Token


class Environment:

    def __init__(self, enclosing: Optional["Environment"] = None):
        self.values = {}
        self.enclosing = enclosing

    def define(self, name: str, value: Any):
        self.values[name] = value

    def assign(self, name: Token, value: Any):
        if name.lexeme in self.values:
            self.values[name.lexeme] = value
        elif self.enclosing:
            self.enclosing.assign(name=name, value=value)
        else:
            raise RunTimeException(
                token=name, message=f"cant assign, Undefined variable: [{name.lexeme}]"
            )

    def assign_at(self, distance: int, name: Token, value: Any):
        self.ancestor(distance=distance).values[name.lexeme] = value

    def get(self, name: Token) -> Any:
        if name.lexeme in self.values:
            return self.values[name.lexeme]
        elif self.enclosing:
            return self.enclosing.get(name)
        else:
            raise RunTimeException(
                token=name, message=f"Can't get. Undefined variable: [{name.lexeme}]"
            )

    def get_at(self, distance: int, name: str):
        return self.ancestor(distance=distance).values[name]

    def ancestor(self, distance: int) -> "Environment":
        environment = self
        for _ in range(distance):
            environment = environment.enclosing

        return environment
