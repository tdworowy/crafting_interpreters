from src.expr import Binary, Literal
from src.parser import Parser
from src.token_ import Token, TokenType


def test_parser():

    minus_token = Token(TokenType.MINUS, lexeme="-", literal=None, line=0)
    expr = Parser(
        tokens=[
            Token(TokenType.NUMBER, lexeme="1", literal=1, line=0),
            minus_token,
            Token(TokenType.NUMBER, lexeme="5", literal=5, line=0),
            Token(TokenType.EOF, lexeme="", literal=None, line=1),
        ]
    ).expression()
    assert expr == Binary(
        left=Literal(value=1), operator=minus_token, right=Literal(value=5)
    )
