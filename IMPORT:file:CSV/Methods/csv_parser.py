# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# The CSVParser class the importer's callers construct.
#
# Owns: the object wrapper around `oaRustCore.csv_parser`. It holds no
# parsing logic of its own by design — the rule stated below as "No Python
# fallback" is the contract, not a note.
#
# Must not: import `oaRustCore` defensively. Unlike the sibling `Entry.py`
# files, this module imports it unguarded and will raise on a machine that has
# not built the extension. That is deliberate: a parser that silently becomes a
# no-op is worse here than one that refuses to load, because its output is a
# record set that looks complete.

# oaFileHandlers/oaFileImportCSV/Methods/csv_parser.py
# Author: Anthony Peter Kuzub
# Version: 20260331.2150.2
#
# Description: Pure Rust CSV parser with Polars support (No Python fallback).

LOCAL_DEBUG = False
from oaRustCore import csv_parser as apkaudio_csvparser_rs


class CSVParser:
    """
    High-performance CSV parser using Rust and Polars.
    MANDATORY Rust implementation.
    """
    @staticmethod
    def convert_csv_unknown(file_path: str):
        if LOCAL_DEBUG:
            print("📄🛠️🔗 [CSV] Using PURE RUST parser (nom/regex).")
        return apkaudio_csvparser_rs.convert_csv_unknown(file_path)

    @staticmethod
    def load_large_csv(file_path: str):
        if LOCAL_DEBUG:
            print("📊🛠️🔗 [CSV] Using PURE RUST loader (Polars).")
        return apkaudio_csvparser_rs.load_large_csv(file_path)
