from src.ast_printer import AstPrinter
from src.parser import Parser
from src.scanner import Scanner


def main():
    source = "5 + (2 * (2 + 3) + 4) + 5"

    scanner = Scanner(source=source)
    tokens = scanner.scan_tokens()

    expr = Parser(tokens=tokens).parse()

    print(AstPrinter().print_ast(expr=expr))


if __name__ == "__main__":
    main()
