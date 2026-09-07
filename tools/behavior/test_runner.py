"""Small synthetic checks for false-success and reference-corruption failure modes."""

import sys

sys.dont_write_bytecode = True

import hashlib
import json
from pathlib import Path
import tempfile
import unittest

import run as runner


class ReferenceContractTests(unittest.TestCase):
    def test_missing_and_wrong_jar_are_blocked(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "synthetic.jar"
            content = b"synthetic reference identity, not an original JAR"
            expected = hashlib.sha1(content).hexdigest()
            with self.assertRaises(runner.Blocked):
                runner.check_file(path, expected)
            path.write_bytes(b"wrong identity")
            with self.assertRaises(runner.Blocked):
                runner.check_file(path, expected)
            path.write_bytes(content)
            runner.check_file(path, expected)

    def test_missing_reordered_and_malformed_results_fail(self):
        cases = [{"case_id": "a"}, {"case_id": "b"}]
        records = [{"schema_version": 1, "case_id": value, "result": {"value": "10"}}
                   for value in ("a", "b")]
        pack = lambda values: "\n".join(json.dumps(x) for x in values)
        self.assertEqual(runner.parse_records(pack(records), cases), records)
        for raw in ("", pack(records[:1]), pack(records[::-1]), pack(records[:1] * 2),
                    "not JSON", pack(records).replace('"value": "10"', '"value": NaN'),
                    pack(records).replace('"value": "10"', '"value": "10", "value": "11"')):
            with self.subTest(raw=raw), self.assertRaises(runner.Failed):
                runner.parse_records(raw, cases)

    def test_mutation_reports_first_field_without_tolerance(self):
        expected = {"events": [{"bits": "00000000", "tick": 1}]}
        actual = {"events": [{"bits": "80000000", "tick": 1}]}
        difference = runner.first_difference(expected, actual)
        self.assertEqual(difference["path"], "result.events[0].bits")
        self.assertIsNotNone(runner.first_difference(1, True))
        self.assertIsNotNone(runner.first_difference(1, 1.0))
        self.assertIsNotNone(runner.first_difference([1, 2], [2, 1]))
        self.assertIsNotNone(runner.first_difference({"a": 1}, {}))
        self.assertIsNone(runner.first_difference(expected, expected))

    def test_capture_requires_matching_identity_and_intact_content(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            inputs = b'{"case_id":"synthetic"}\n'
            output = '{"schema_version":1,"case_id":"synthetic","result":{"value":"10"}}\n'
            identity = {"inputs_sha256": runner.digest(inputs), "oracle": "synthetic"}
            (path / "inputs.jsonl").write_bytes(inputs)
            (path / "outputs.jsonl").write_text(output, encoding="utf-8")
            runner.save(path / "manifest.json", {"identity": identity,
                                                 "outputs_sha256": runner.digest(output.encode())})
            cases = [{"case_id": "synthetic"}]
            runner.load_capture(path, identity, cases)
            with self.assertRaises(runner.Blocked):
                runner.load_capture(path, dict(identity, oracle="different"), cases)
            (path / "outputs.jsonl").write_text(output.replace('"10"', '"11"'), encoding="utf-8")
            with self.assertRaises(runner.Blocked):
                runner.load_capture(path, identity, cases)

    def test_failed_and_hung_processes_cannot_produce_success(self):
        with tempfile.TemporaryDirectory() as directory:
            cwd = Path(directory)
            with self.assertRaises(runner.Failed):
                runner.execute([sys.executable, "-c", "raise SystemExit(3)"], cwd, "failure")
            with self.assertRaises(runner.Failed):
                runner.execute([sys.executable, "-c", "import time; time.sleep(5)"], cwd, "timeout", timeout=0.05)


if __name__ == "__main__":
    unittest.main()
