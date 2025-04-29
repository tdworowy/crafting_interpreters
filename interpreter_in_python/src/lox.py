from src.interpreter import Interpreter
from src.parser import Parser
from src.scanner import Scanner


class Lox:
    def __init__(self):
        self.interpreter = Interpreter()
        self.had_error = False

    def run(self, source: str):
        scanner = Scanner(source)
        self.had_error = scanner.had_error
        tokens = scanner.scan_tokens()

        parser = Parser(tokens)
        expr = parser.parse()

        value = self.interpreter.interpret(expr)
        self.had_error = self.interpreter.had_error

        print(value)
