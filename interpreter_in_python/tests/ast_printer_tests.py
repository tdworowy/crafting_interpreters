from src.ast_printer import AstPrinter
from src.expr import Assign, Binary, Grouping, Literal, Unary
from src.token_ import Token, TokenType


def test_ast_printer_assign():
    result = AstPrinter().print_ast(
        expr=Assign(
            name=Token(
                token_type=TokenType.IDENTIFIER, lexeme="x", literal=None, line=1
            ),
            value=Literal(value="Test"),
        )
    )
    assert result == "(= x Test)"


def test_ast_printer_binary():
    result = AstPrinter().print_ast(
        expr=Binary(
            left=Literal(value=2),
            operator=Token(
                token_type=TokenType.LESS_EQUAL, lexeme="<=", literal=None, line=1
            ),
            right=Literal(value=3),
        )
    )
    assert result == "(<= 2 3)"


def test_expression():
    expression = Binary(
        left=Unary(
            operator=Token(
                token_type=TokenType.MINUS, lexeme="-", literal=None, line=1
            ),
            right=Literal(123),
        ),
        operator=Token(token_type=TokenType.STAR, lexeme="*"),
        right=Grouping(Literal(45.67)),
    )

    assert AstPrinter().print_ast(expr=expression) == "(* (- 123) (group 45.67))"
