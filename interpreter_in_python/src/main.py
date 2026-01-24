from pathlib import Path

from src.lox import Lox

current_dir = Path(__file__).parent.resolve()


def main():
    with (current_dir / "../../tests/lox_scripts/break_test.lox").open(mode="r") as f:
        source = f.read()
        Lox().run(source=source, repl=False)


def repl():
    lox = Lox()
    while True:
        source = input()
        result = lox.run(source=source, repl=True)
        if result:
            print(result)


if __name__ == "__main__":
    main()
    # repl()
