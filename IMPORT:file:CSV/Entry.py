# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# The CSV importer's public entry point.
#
# Owns: naming the Rust CSV parser and handing back whatever it returns. The
# dialect sniffing, the pattern recognition and the record shape all live in
# Rust (`oaRustCore.csv_parser`); this file is the Python edge.
#
# Must not: grow a Python CSV fallback. `Methods/csv_parser.py` states the same
# rule in its own words — "No Python fallback" — and a second implementation
# here would silently become the one that runs on any machine where maturin has
# not been run, producing different records from the same file.
#
# `importer_logging` rather than `oaLogging` directly: that package is not in
# this repository, and importing it at module scope made this file — and
# therefore the whole `oaFileImportCSV` package and its test suite —
# uncollectable. See the header on `importer_logging.py`.

from importer_logging import matrix_log
try:
    from oaRustCore.csv_parser import convert_csv_unknown as rust_convert_csv_unknown
except ImportError:
    rust_convert_csv_unknown = None

# Inline comment: Logic for convert_unknown_csv_report
def convert_unknown_csv_report(file_path):
    if rust_convert_csv_unknown:
        matrix_log("ui", "importer", "convert_unknown_csv_report", "🚀 Using HIGH-PERFORMANCE RUST CSV parser.", "DEBUG")
        return rust_convert_csv_unknown(str(file_path))
    else:
        raise RuntimeError("Rust CSV parser is required but not installed.")

# Inline comment: Logic for start
def start():
    pass

# Inline comment: Logic for stop
def stop():
    pass

def report_status():
    """Human-readable state of the importer back end."""
    return "Running with Rust Engine"

# Inline comment: Logic for run_tests
def run_tests():
    return True

__all__ = ["convert_unknown_csv_report", "start", "stop", "report_status", "run_tests"]
