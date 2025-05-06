from src.lox import Lox


def main():
    with open("../example2.lox") as f:
        source = f.read()
        Lox().run(source)


if __name__ == "__main__":
    main()
