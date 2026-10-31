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
# The logging call and the importer lifecycle come from `apk_wireless_import`
# (SRC/python/ in this repository, shared with the IAS, SoundBase-PDF and
# Wireless Workbench edges), not from NODE.BareMetal's `importer_logging`:
# this edge no longer needs another repository on PYTHONPATH to import.

from apk_wireless_import import Lifecycle, log

try:
    from oaRustCore.csv_parser import convert_csv_unknown as rust_convert_csv_unknown
except ImportError:
    rust_convert_csv_unknown = None

# "live" when the Rust parser is reachable, "unavailable" when it is not.
STATUS = "live" if rust_convert_csv_unknown else "unavailable"


def convert_unknown_csv_report(file_path):
    if rust_convert_csv_unknown:
        log("convert_unknown_csv_report", "🚀 Using HIGH-PERFORMANCE RUST CSV parser.")
        return rust_convert_csv_unknown(str(file_path))
    raise RuntimeError("Rust CSV parser is required but not installed.")


def _self_test():
    """The parser answers at all: a two-row zone/group file through it."""
    import os
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "self_test.csv")
        with open(path, "w") as f:
            f.write("ZONE,GROUP,DEVICE,NAME,FREQ\nA,G1,D,Lead,566.600\n")
        headers, rows = convert_unknown_csv_report(path)
    return bool(headers) and len(rows) == 1


_life = Lifecycle(STATUS, self_test=_self_test)
start, stop, status, run_tests = _life.start, _life.stop, _life.status, _life.run_tests


def report_status():
    """Human-readable state of the importer back end — the STATUS word, no more.

    It used to answer "Running with Rust Engine" whether or not the engine was built."""
    return STATUS


__all__ = ["convert_unknown_csv_report", "STATUS", "start", "stop", "status", "report_status", "run_tests"]
