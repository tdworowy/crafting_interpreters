from src.lox_callable import LoxCallable
from src.lox_instance import LoxInstance


class LoxClass(LoxCallable):
    def call(self, interpreter: "Interpreter", arguments: list):
        instance = LoxInstance(klass=self)
        return instance

    def arity(self) -> int:
        return 0

    def __init__(self, name: str):
        self.name = name

    def __str__(self):
        return self.name

    def __repr__(self):
        return self.name
