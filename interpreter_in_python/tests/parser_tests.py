from src.expr import Binary, Literal
from src.parser import Parser
from src.stmt import Expression
from src.token_ import Token, TokenType


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
