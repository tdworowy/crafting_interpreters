from typing import Optional

from src.lox_callable import LoxCallable
from src.lox_function import LoxFunction
from src.lox_instance import LoxInstance


class LoxClass(LoxCallable, LoxInstance):

    def __init__(
        self,
        meta_class: Optional["LoxClass"],
        name: str,
        methods: dict[str, LoxFunction],
        super_class: Optional["LoxClass"] = None,
    ):
        super().__init__(klass=meta_class)
        self.name = name
        self.methods = methods
        self.super_class = super_class

    def __str__(self):
        return self.name

    def __repr__(self):
        return self.name

    def call(self, interpreter: "Interpreter", arguments: list):
        instance = LoxInstance(klass=self)
        initializer = self.find_method(name="init")
        if initializer:
            initializer.bind(instance=instance).call(
                interpreter=interpreter, arguments=arguments
            )
        return instance

    def arity(self) -> int:
        initializer = self.find_method(name="init")
        if initializer:
            return initializer.arity()
        else:
            return 0

    def find_method(self, name: str) -> LoxFunction | None:
        method = self.methods.get(name, None)
        if method:
            return method
        elif self.super_class:
            return self.super_class.find_method(name=name)
