# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# FileReaders/from_csv_unknown.py
#
# Best-effort CSV conversion for unknown header formats, by the Rust parser.
#
# There is no Python parser and no fallback: `oaRustCore.csv_parser` (built from
# NODE.BareMetal/SRC/Core/CORE:engine:RUST/PARSER/csv_parser.rs) is the one
# implementation. Without it this raises — the same rule as `Entry.py` and
# `Methods/csv_parser.py`. It used to log "Falling back to slow Python parsing"
# and then call a name that was never bound, so the warning described a fallback
# that did not exist and the call died with a NameError.

import inspect

from apk_wireless_import import log

try:
    from oaRustCore.csv_parser import convert_csv_unknown as rust_convert_csv_unknown
except ImportError:
    rust_convert_csv_unknown = None


def convert_unknown_csv_report(file_path):
    """(standard_headers, rows) for a CSV with unknown headers, by the mandatory Rust parser.

    Raises RuntimeError when the Rust parser is not installed, and the parser's own
    OSError for a missing or malformed file."""
    here = inspect.currentframe().f_code.co_name
    if rust_convert_csv_unknown is None:
        log(here, "🚀❌ [CSV] Rust CSV parser missing — refusing; there is no Python parser.", "ERROR")
        raise RuntimeError("Rust CSV parser (oaRustCore.csv_parser) is required but not installed.")
    log(here, f"▶️ Rust best-effort CSV conversion for: {file_path}", "DEBUG")
    try:
        return rust_convert_csv_unknown(str(file_path))
    except Exception as e:
        log(here, f"🚀❌ [FATAL] Rust CSV conversion failed: {e}", "ERROR")
        raise
