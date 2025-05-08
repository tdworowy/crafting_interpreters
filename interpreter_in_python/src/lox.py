from src.interpreter import Interpreter
from src.parser import Parser
from src.scanner import Scanner


class Lox:
    def __init__(self):
        self.interpreter = Interpreter()
        self.had_error = False

    def run(self, source: str, repl: bool) -> str | None:
        scanner = Scanner(source=source)
        self.had_error = scanner.had_error
        tokens = scanner.scan_tokens()

        parser = Parser(tokens=tokens)
        if repl:
            to_interpret = parser.parse_repl()
        else:
            to_interpret = parser.parse()

        result = self.interpreter.interpret(to_interpret=to_interpret)
        self.had_error = self.interpreter.had_error
        return result
