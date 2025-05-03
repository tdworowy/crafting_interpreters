from dataclasses import dataclass
from typing import Any

from src.run_time_exception import RuneTimeException
from src.token_ import Token


@dataclass
class Environment:
    values: dict[str, Any]

    def define(self, name: str, value: Any):
        self.values[name] = value

    def get(self, name: Token):
        if name.lexeme in self.values:
            return self.values[name.lexeme]
        else:
            raise RuneTimeException(
                token=name, message=f"Undefined variable: [{name.lexeme}]"
            )
