from src.scanner import Scanner, TokenType


def test_scan_tokens():
    source = """+ - <=
    >= != {}
    * ()
    /
    // 
    /"""
    expected_tokens = [
        TokenType.PLUS,
        TokenType.MINUS,
        TokenType.LESS_EQUAL,
        TokenType.GREATER_EQUAL,
        TokenType.BANG_EQUAL,
        TokenType.LEFT_BRACE,
        TokenType.RIGHT_BRACE,
        TokenType.STAR,
        TokenType.LEFT_PAREN,
        TokenType.RIGHT_PAREN,
        TokenType.SLASH,
        TokenType.SLASH,
        TokenType.EOF,
    ]
    scanner = Scanner(source=source)
    tokens = scanner.scan_tokens()

    assert scanner.line == len(source.split("\n"))
    tokens_types = [t.token_type for t in tokens]
    assert tokens_types == expected_tokens
