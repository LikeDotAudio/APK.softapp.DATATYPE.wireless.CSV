# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# FileReaders/from_csv_unknown.py
#
# Best-effort CSV parser for unknown header formats.
#
# Author: Anthony Peter Kuzub
# Version: 20260331.2240.1

import inspect

from loguru import logger

from importer_logging import matrix_log

# --- Constants ---
VERSION = "20260331.2240.1"

try:
    from oaRustCore.csv_parser import convert_csv_unknown as rust_convert_csv_unknown
    HAS_RUST_CSV = True
except ImportError:
    logger.warning("🚀⚠️ [CSV] Rust CSV Parser missing. Falling back to slow Python parsing.")
    HAS_RUST_CSV = False

# Inline comment: Logic for convert_unknown_csv_report
def convert_unknown_csv_report(file_path):
    """
    Performs a 'best-effort' conversion of a CSV file with unknown headers
    to the standardized marker report format using the mandatory Rust parser.
    """
    matrix_log("ui", "importer", inspect.currentframe().f_code.co_name if "inspect" in globals() else "unknown", f"▶️ Starting best-effort CSV conversion for: {file_path}", "DEBUG")

    try:
        standard_headers, processed_rows = rust_convert_csv_unknown(file_path)
        matrix_log("ui", "importer", inspect.currentframe().f_code.co_name if "inspect" in globals() else "unknown", "🚀 Using HIGH-PERFORMANCE RUST CSV parser.", "DEBUG")
        return standard_headers, processed_rows
    except Exception as e:
        matrix_log("ui", "importer", inspect.currentframe().f_code.co_name if "inspect" in globals() else "unknown", f"🚀❌ [FATAL] Rust CSV conversion failed: {e}", "ERROR")
        raise e
