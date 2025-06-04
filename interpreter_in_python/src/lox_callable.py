from abc import ABC, abstractmethod


class LoxCallable(ABC):
    @abstractmethod
    def call(self, interpreter: "Interpreter", arguments: list):
        pass

    @abstractmethod
    def arity(self) -> int:
        pass
