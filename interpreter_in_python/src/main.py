from src.lox import Lox


def main():
    source = "print 5 + (2 * (2 + 3) + 4) + 6;\nprint 2 * 2;\nvar x = 12;\nvar y = 12;\nprint x;\nprint x + y;"

    Lox().run(source)


if __name__ == "__main__":
    main()
