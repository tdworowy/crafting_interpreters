from src.ast_printer import AstPrinter
from src.expr import Assign, Binary, Grouping, Literal, Unary
from src.interpreter import Interpreter
from src.token_ import Token, TokenType


def test_two_plus_two():
    expr = Binary(
        left=Literal(value=2),
        operator=Token(token_type=TokenType.PLUS, lexeme="=", literal=None, line=1),
        right=Literal(value=2),
    )
    assert Interpreter().evaluate(expr=expr) == 4
