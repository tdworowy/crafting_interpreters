from src.lox_callable import LoxCallable
from src.lox_function import LoxFunction
from src.lox_instance import LoxInstance


class LoxClass(LoxCallable):

    def __init__(self, name: str, methods: dict[str, LoxFunction]):
        self.name = name
        self.methods = methods

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
        return self.methods.get(name, None)
