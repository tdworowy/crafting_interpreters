from src.token_ import Token


class RunTimeException(RuntimeError):
    def __init__(self, token: Token, message):
        self.token = token
        self.message = message
        super().__init__(message)
