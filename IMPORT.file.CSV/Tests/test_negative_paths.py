# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# Tests/test_negative_paths.py — the sad paths: a missing parser, a missing file,
# a malformed row. Each must raise; none may return an empty, complete-looking result.
#
# These were two `pass` bodies around commented-out calls to a function that does
# not exist, so they passed without testing anything.

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from FileReaders import from_csv_unknown

try:
    from oaRustCore import csv_parser as _rust  # noqa: F401
    HAVE_RUST = True
except ImportError:
    HAVE_RUST = False


class TestNegativePaths(unittest.TestCase):
    def setUp(self):
        self.test_directory = tempfile.TemporaryDirectory()
        self.root = Path(self.test_directory.name)

    def tearDown(self):
        self.test_directory.cleanup()

    def test_no_rust_parser_refuses_rather_than_falls_back(self):
        """Without the Rust parser there is no answer, not a slower or emptier one."""
        with mock.patch.object(from_csv_unknown, "rust_convert_csv_unknown", None):
            with self.assertRaises(RuntimeError) as caught:
                from_csv_unknown.convert_unknown_csv_report(self.root / "any.csv")
        self.assertIn("Rust CSV parser", str(caught.exception))

    @unittest.skipUnless(HAVE_RUST, "oaRustCore not built")
    def test_csv_ingestion_missing_file(self):
        with self.assertRaises(OSError):
            from_csv_unknown.convert_unknown_csv_report(self.root / "does_not_exist.csv")

    @unittest.skipUnless(HAVE_RUST, "oaRustCore not built")
    def test_csv_ingestion_malformed_row(self):
        """A row with the wrong number of fields is an error, not a silently short record."""
        path = self.root / "malformed.csv"
        path.write_text("ZONE,FREQ\nA\nB,500,extra\n")
        with self.assertRaises(OSError):
            from_csv_unknown.convert_unknown_csv_report(path)


if __name__ == "__main__":
    unittest.main()
