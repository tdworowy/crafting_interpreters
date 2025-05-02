from src.lox import Lox


def main():
    source = "print 5 + (2 * (2 + 3) + 4) + 6;\nprint 2 * 2;"

    Lox().run(source)


if __name__ == "__main__":
    main()
