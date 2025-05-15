from functools import cache

import pytest

from src.lox import Lox


@cache
def fib(n: int) -> int:
    a = 0
    b = 1

    if n == 0:
        return 0
    elif n == 1:
        return b
    else:
        for i in range(1, n):
            c = a + b
            a = b
            b = c
        return b


def expected_data() -> str:
    return "\n".join([f"{str(fib(i))}.0" for i in range(26)]) + "\n"


def test_fibonacci(capsys: pytest.CaptureFixture[str]):
    with open("fib.lox") as f:
        source = f.read()
        Lox().run(source=source, repl=True)
        captured = capsys.readouterr()
        assert captured.out == expected_data()


def test_break(capsys: pytest.CaptureFixture[str]):
    with open("break_test.lox") as f:
        source = f.read()
        Lox().run(source=source, repl=True)
        captured = capsys.readouterr()
        assert captured.out == "After loop\n"
