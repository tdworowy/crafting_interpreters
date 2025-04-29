from src.lox import Lox


def main():
    source = "5 + (2 * (2 + 3) + 4) + 6"

    Lox().run(source)


if __name__ == "__main__":
    main()
