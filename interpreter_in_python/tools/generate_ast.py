def define_ast(output_path: str, types: list[str]):
    with open(output_path, "w") as output:

        output.write("from abc import ABC\n")
        output.write("from dataclasses import dataclass\n")
        output.write("from src.token import Token\n")
        output.write("class Expr(ABC):\n\tpass\n")
        for _type in types:
            name, fields = _type.split(":")
            fields = fields.strip().split(",")
            class_body = f"@dataclass\nclass {name.strip()}(Expr):\n" ""
            for field in fields:
                field = field.strip().split(" ")
                class_body += f"\t{field[1].strip()}: \t{field[0].strip()}\n"
            output.write(class_body)


if __name__ == "__main__":
    exper = [
        "Assign   : Token name, Expr value",
        "Binary   : Expr left, Token operator, Expr right",
        "Call     : Expr callee, Token paren, list[Expr] arguments",
        "Get      : Expr object, Token name",
        "Grouping : Expr expression",
        "Literal  : str value",
        "Logical  : Expr left, Token operator, Expr right",
        "Set      : Expr object, Token name, Expr value",
        "Super    : Token keyword, Token method",
        "This     : Token keyword",
        "Unary    : Token operator, Expr right",
        "Variable : Token name",
    ]
    Stmt = [
        "Block      : list[Stmt] statements",
        "Class      : Token name, Expr.Variable superclass, list[Stmt].Function> methods",
        "Function   : Token name, list[Token] params, list[Stmt] body",
        "If         : Expr condition, Stmt thenBranch, Stmt elseBranch",
        "Print      : Expr expression",
        "Print      : Expr expression",
        "Return     : Token keyword, Expr value",
        "Var        : Token name, Expr initializer"
        "Var        : Token name, Expr initializer",
        "While      : Expr condition, Stmt body",
    ]
    define_ast("expr.py", exper)
