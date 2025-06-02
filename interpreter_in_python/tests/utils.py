from typing import Any


def compare_ast(node1: Any, node2: Any, ignore_types: set = None) -> bool:
    if ignore_types is None:
        ignore_types = set()

    if any(isinstance(node1, t) or isinstance(node2, t) for t in ignore_types):
        return True

    if type(node1) is not type(node2):
        return False

    if isinstance(node1, (int, float, str, bool)):
        return node1 == node2

    if isinstance(node1, list):
        if len(node1) != len(node2):
            return False
        return all(compare_ast(n1, n2, ignore_types) for n1, n2 in zip(node1, node2))

    if isinstance(node1, dict):
        if node1.keys() != node2.keys():
            return False
        return all(compare_ast(node1[k], node2[k], ignore_types) for k in node1)

    try:
        attrs1 = vars(node1)
        attrs2 = vars(node2)
        if attrs1.keys() != attrs2.keys():
            return False
        all_check = all(compare_ast(attrs1[k], attrs2[k], ignore_types) for k in attrs1)
        if all_check:
            return True
        else:
            print("\n")
            print(f"attrs1:{attrs1}")
            print(f"attrs2:{attrs2}")
            return False
    except TypeError:
        return True
