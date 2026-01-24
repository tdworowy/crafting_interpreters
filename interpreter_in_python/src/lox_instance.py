from typing import Any

from src.return_exception import ReturnException
from src.token_ import Token


class LoxInstance:

    def __init__(self, klass: "LoxClass"):
        self.klass = klass
        self.fields = {}

    def get(self, name: Token) -> Any:
        if name.lexeme in self.fields:
            return self.fields[name.lexeme]

        method = self.klass.find_method(name=name.lexeme)
        if method:
            return method.bind(self)

        raise ReturnException(token=name, message=f"Undefined property {name.lexeme}.")

    def set(self, name: Token, value: Any):
        self.fields[name.lexeme] = value

    def __str__(self):
        return f"{self.klass}_instance"

    def __repr__(self):
        return f"{self.klass}_instance"
