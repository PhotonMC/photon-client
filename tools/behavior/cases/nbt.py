"""The `nbt` family: original synthetic inputs."""

from cases.helpers import *


def nbt_cases():
    prims = {
        "byte": ("byte", "127"),
        "short": ("short", "-12345"),
        "int": ("int", "2147483647"),
        "long": ("long", "-9223372036854775808"),
        "float": ("float", "3f800000"),
        "double": ("double", "bff0000000000000"),
        "string": ("string", "hi"),
    }
    for label, (kind, value) in prims.items():
        yield {"case_id": f"nbt/write/{label}", "family": "nbt",
               "operation": "write", "prefix_hex": "a55a",
               "fields": [{"key": "k", "type": kind, "value": value}]}
    yield {"case_id": "nbt/write/empty", "family": "nbt",
           "operation": "write", "prefix_hex": "a55a", "fields": []}
    yield {"case_id": "nbt/write/null", "family": "nbt",
           "operation": "write", "prefix_hex": "a55a",
           "null_root": True, "fields": []}
    trees = {
        "nested": {"type": "compound", "value": {
            "c": {"type": "compound", "value": {
                "b": {"type": "byte", "value": "2"}}}}},
        "list-int": {"type": "compound", "value": {
            "l": {"type": "list", "value": [
                {"type": "int", "value": "1"},
                {"type": "int", "value": "-2"},
                {"type": "int", "value": "300"}]}}},
        "list-string": {"type": "compound", "value": {
            "l": {"type": "list", "value": [
                {"type": "string", "value": "a"},
                {"type": "string", "value": "bc"}]}}},
        "empty-list": {"type": "compound", "value": {
            "e": {"type": "list", "value": []}}},
        "byte-array": {"type": "compound", "value": {
            "a": {"type": "byte-array", "value": ["0", "127", "-128"]}}},
        "int-array": {"type": "compound", "value": {
            "a": {"type": "int-array", "value": ["0", "-1", "2147483647"]}}},
        "long-array": {"type": "compound", "value": {
            "a": {"type": "long-array", "value": ["0", "-9223372036854775808"]}}},
        "list-compound": {"type": "compound", "value": {
            "l": {"type": "list", "value": [
                {"type": "compound", "value": {"x": {"type": "int", "value": "1"}}},
                {"type": "compound", "value": {"x": {"type": "int", "value": "2"}}}]}}},
    }
    for label, tree in trees.items():
        yield {"case_id": f"nbt/write-tree/{label}", "family": "nbt",
               "operation": "write", "prefix_hex": "a55a", "tree": tree}
    reads = {
        "empty": "0a0055aa",
        "null": "0055aa",
        "byte": "0a0100016b010055aa",
        "string": "0a08000161000268690055aa",
        "nested": "0a0a0001630100016202000055aa",
        "list-int": "0a0900016c03000000030000000100000002000000030055aa",
        "empty-list": "0a0900016500000000000055aa",
        "non-compound": "030102030455aa",
        "truncated-tag": "0a01",
        "truncated-body": "0a0100016b",
        "truncated-nested": "0a0a000163",
        "short-nested": "0a0a000163010001620200",
        "missing-payload": "0a",
        "empty-input": "",
    }
    for label, framed in reads.items():
        yield {"case_id": f"nbt/read/{label}", "family": "nbt",
               "operation": "read", "prefix_hex": "a55a", "hex": framed}
