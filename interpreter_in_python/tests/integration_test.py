from functools import cache
from pathlib import Path

import pytest
from src.lox import Lox

current_dir = Path(__file__).parent.resolve()


@cache
def fib(n: int) -> int:
    a = 0
    b = 1

    if n == 0:
        return 0
    elif n == 1:
        return b
    else:
        for _ in range(1, n):
            c = a + b
            a = b
            b = c
        return b


@cache
def expected_data() -> str:
    return "\n".join([f"{str(fib(i))}.0" for i in range(26)]) + "\n"


tests = [
    ("lox_scripts/break_test.lox", "After loop\n"),
    ("lox_scripts/lambda1.lox", "lambda works\n"),
    ("lox_scripts/lambda2.lox", "lambda works\n"),
    ("lox_scripts/class1.lox", "instance property works\n"),
    ("lox_scripts/class2.lox", "Staff\n"),
    ("lox_scripts/class3.lox", "thisStaff\n"),
    ("lox_scripts/class4.lox", "constructors works\n"),
    ("lox_scripts/class5.lox", "class method works\n"),
    ("lox_scripts/class6.lox", "Inheritance works\n"),
    ("lox_scripts/class7.lox", "Inheritance works\n"),
    ("lox_scripts/fib.lox", expected_data()),
    ("lox_scripts/fib_function.lox", expected_data()),
]


@pytest.mark.parametrize(
    "file_name,expected_result",
    tests,
)
def test_lox(file_name: str, expected_result: str, capsys: pytest.CaptureFixture[str]):
    with (current_dir / file_name).open(mode="r") as f:
        source = f.read()
        Lox().run(source=source, repl=True)
        captured = capsys.readouterr()
        assert captured.out == expected_result
