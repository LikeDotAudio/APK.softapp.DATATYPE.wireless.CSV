# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# Tests/test_rust_csv_parser.py — the Rust CSV parser through the Python edge.
#
# Imports the reader by its path in this package (see conftest.py); skips, naming
# why, when the oaRustCore extension has not been built on this machine.

import unittest
from pathlib import Path

from FileReaders.from_csv_unknown import convert_unknown_csv_report

try:
    from oaRustCore import csv_parser as _rust  # noqa: F401
    HAVE_RUST = True
except ImportError:
    HAVE_RUST = False


@unittest.skipUnless(HAVE_RUST, "oaRustCore not built — maturin build NODE.BareMetal/SRC/Core")
class TestRustCSVParser(unittest.TestCase):
    def setUp(self):
        self.asset_path = Path(__file__).parent / "Assets" / "Basic_CSV_venue.csv"
        self.assertTrue(self.asset_path.exists(), f"sample asset missing: {self.asset_path}")

    def test_rust_csv_parsing(self):
        headers, data = convert_unknown_csv_report(str(self.asset_path))

        self.assertGreater(len(data), 0)
        for want in ("ZONE", "NAME", "FREQ_MHZ"):
            self.assertIn(want, headers)
        for row in data:
            self.assertEqual(len(row), len(headers))
        self.assertAlmostEqual(data[0]["FREQ_MHZ"], 472.55)


if __name__ == "__main__":
    unittest.main()
