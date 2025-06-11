from src.lox import Lox


def main():
    with open("../tests/lox_scripts/lambda2.lox") as f:
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
