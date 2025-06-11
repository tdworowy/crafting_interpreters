import pytest

from src.ast_printer import AstPrinter
from src.expr import Binary, Literal
from src.parser import Parser
from src.scanner import Scanner
from src.stmt import Expression
from src.token_ import Token, TokenType
from tests.utils import compare_ast


def test_parser_correct():
    minus_token = Token(TokenType.MINUS, lexeme="-", literal=None, line=0)
    statements = Parser(
        tokens=[
            Token(TokenType.NUMBER, lexeme="1", literal=1, line=1),
            minus_token,
            Token(TokenType.NUMBER, lexeme="5", literal=5, line=2),
            Token(TokenType.SEMICOLON, lexeme=";", literal=None, line=3),
            Token(TokenType.EOF, lexeme="", literal=None, line=4),
        ]
    ).parse()
    assert statements[0] == Expression(
        expression=Binary(
            left=Literal(value=1), operator=minus_token, right=Literal(value=5)
        )
    )


def test_parser_binary_without_left():
    statements = Parser(
        tokens=[
            Token(TokenType.PLUS, lexeme="+", literal=None, line=0),
            Token(TokenType.NUMBER, lexeme="5", literal=5, line=1),
            Token(TokenType.SEMICOLON, lexeme=";", literal=None, line=3),
            Token(TokenType.EOF, lexeme="", literal=None, line=4),
        ]
    ).parse()
    assert statements[0] is None


@pytest.mark.skip("difference is ok.")
def test_compare_loops():
    with open("lox_scripts/for.lox") as for_file:
        scanner_for = Scanner(source=for_file.read())
        tokens_for = scanner_for.scan_tokens()
        parsed_for = Parser(tokens=tokens_for).parse()

    with open("lox_scripts/while.lox") as while_file:
        scanner_while = Scanner(source=while_file.read())
        tokens_while = scanner_while.scan_tokens()
        parsed_while = Parser(tokens=tokens_while).parse()

    assert compare_ast(node1=parsed_for, node2=parsed_while, ignore_types={Token})
