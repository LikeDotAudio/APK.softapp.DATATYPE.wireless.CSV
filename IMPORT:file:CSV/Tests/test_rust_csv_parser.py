# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# oaFileHandlers/oaFileImportCSV/Tests/test_rust_csv_parser.py
#
# Tests for the CSV Parser (Rust implementation).
#
# Author: Anthony Peter Kuzub
# Version: 20260401.1000.1

import unittest
from pathlib import Path

from oaFileImportCSV.FileReaders.from_csv_unknown import convert_unknown_csv_report


class TestRustCSVParser(unittest.TestCase):
    def setUp(self):
        # Find the sample asset
        self.asset_path = Path(__file__).parent / "Assets" / "Basic_CSV_venue.csv"
        if not self.asset_path.exists():
            # No absolute fallback: the old one pointed at a repo layout that no
            # longer exists, so it turned a missing asset into a confusing error
            # about someone else's home directory. Fail with the path we wanted.
            self.skipTest(f"sample asset not found: {self.asset_path}")


    def test_rust_csv_parsing(self):
        """Test CSV parsing with the mandatory Rust backend."""
        try:
            from oaRustCore import csv_parser as apkaudio_csvparser_rs
        except ImportError:
            self.skipTest("Rust apkaudio_csvparser_rs not installed.")

        headers, data = convert_unknown_csv_report(str(self.asset_path))

        self.assertGreater(len(data), 0)
        # Verify we have some standard headers (e.g., ZONE, NAME, FREQ_MHZ)
        self.assertIn("ZONE", headers)
        self.assertIn("NAME", headers)
        self.assertIn("FREQ_MHZ", headers)

        # Verify data structure
        for row in data:
            self.assertEqual(len(row), len(headers))

if __name__ == "__main__":
    unittest.main()
