from src.run_time_exception import RunTimeException
from src.token_ import Token


class ReturnException(RunTimeException):
    def __init__(self, token: Token, message):
        super().__init__(token=token, message=message)
