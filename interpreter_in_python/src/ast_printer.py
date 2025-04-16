from src.expr import Visitor, T, Assign, Expr, Literal
from src.token_ import Token, TokenType


class AstPrinter(Visitor):
    def visit_assign_expr(self, expr: "Assign") -> T:
        return self.parenthesize2("=", expr.name.lexeme, expr.value)

    def parenthesize2(self, name: str, *parts) -> str:
        result = f"({name}"
        result += self.transform(parts)
        result += ")"
        return result

    def transform(self, parts: tuple[any, ...]):
        result = ""
        for part in parts:
            match part:
                case Expr():
                    result += part.accept(self)
                case _:
                    result += part
        return result


if __name__ == "__main__":
    AstPrinter().visit_assign_expr(
        Assign(
            name=Token(TokenType.IDENTIFIER, lexeme="X", literal=None, line=1),
            value=Literal(value="Test"),
        )
    )

# TODO finish it
# bassed on https://github.com/munificent/craftinginterpreters/blob/master/java/com/craftinginterpreters/lox/AstPrinter.java
