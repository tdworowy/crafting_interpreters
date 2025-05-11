import pytest

from src.expr import Binary, Literal
from src.interpreter import Interpreter
from src.run_time_exception import RunTimeException
from src.token_ import Token, TokenType


def test_two_plus_two():
    expr = Binary(
        left=Literal(value=2),
        operator=Token(token_type=TokenType.PLUS, lexeme="+", literal=None, line=1),
        right=Literal(value=2),
    )
    assert Interpreter().evaluate(expr=expr) == 4


def test_incorrect_plus():
    with pytest.raises(RunTimeException):
        expr = Binary(
            left=Literal(value=2),
            operator=Token(token_type=TokenType.PLUS, lexeme="+", literal=None, line=1),
            right=Literal(value="test"),
        )
        Interpreter().evaluate(expr=expr)


def test_incorrect_minus():
    with pytest.raises(RunTimeException):
        expr = Binary(
            left=Literal(value=2),
            operator=Token(
                token_type=TokenType.MINUS, lexeme="-", literal=None, line=1
            ),
            right=Literal(value="test"),
        )
        Interpreter().evaluate(expr=expr)


def test_divide_by_zero():
    with pytest.raises(RunTimeException):
        expr = Binary(
            left=Literal(value=2),
            operator=Token(
                token_type=TokenType.SLASH, lexeme="/", literal=None, line=1
            ),
            right=Literal(value=0),
        )
        Interpreter().evaluate(expr=expr)
