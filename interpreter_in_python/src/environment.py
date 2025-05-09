from dataclasses import dataclass, field
from typing import Any

from src.run_time_exception import RuneTimeException
from src.token_ import Token


@dataclass
class Environment:
    values: dict[str, Any]
    enclosing: "Environment" = None

    def define(self, name: str, value: Any):
        self.values[name] = value

    def assign(self, name: Token, value: Any):
        if name.lexeme in self.values:
            self.values[name.lexeme] = value
        elif self.enclosing:
            self.enclosing.assign(name=name, value=value)
        else:
            raise RuneTimeException(
                token=name, message=f"Undefined variable: [{name.lexeme}]"
            )

    def get(self, name: Token):
        if name.lexeme in self.values:
            return self.values[name.lexeme]
        elif self.enclosing:
            return self.enclosing.get(name)
        else:
            raise RuneTimeException(
                token=name, message=f"Undefined variable: [{name.lexeme}]"
            )
