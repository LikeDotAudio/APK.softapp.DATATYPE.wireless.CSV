# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
"""What the wireless file importers' Python edges share — one copy.

    from apk_wireless_import import MARKER_HEADERS, Lifecycle, log

Owns:
  * MARKER_HEADERS — the marker table's header row, the data contract with the
    Data Migration frames. The IAS, SoundBase-PDF and Wireless Workbench edges
    each carried their own copy of this list; nothing kept the three in step.
  * Lifecycle — the importer interface's start/stop/status/run_tests, for an
    importer that holds nothing between calls. `run_tests()` is True only when
    the importer is "live" AND its own self test passes: never True for having
    run nothing (the SoundBase PDF stub's returned True unconditionally).
  * log — the importers' logging call. The real `oaLogging` matrix gate when
    that package is importable, a no-op otherwise; LOGGING_BACKEND says which.
    The edges used to import `importer_logging` from NODE.BareMetal/SRC/Core,
    which made each of them unimportable without that other repository on
    PYTHONPATH.

Consumers name it by package — `apk-wireless-import`, this folder, at a git rev
of DATATYPE.wireless.CSV (see each edge's requirements.txt) — never by path.

Must not: parse anything. The parsers are Rust; this is only what the Python
edges around them would otherwise copy.
"""

MARKER_HEADERS = ["ZONE", "GROUP", "DEVICE", "NAME", "FREQ_MHZ", "PEAK"]

# The three words an importer may declare (Tests/test_handler_status.py in
# NODE.BareMetal holds every edge to them): "live" — its engine answers;
# "stub" — the engine loads, the parser is unwritten; "unavailable" — the
# engine is not built here.
STATUS_WORDS = ("live", "stub", "unavailable")

try:
    from oaLogging.Methods.matrix_gate import matrix_log as _matrix_log

    LOGGING_BACKEND = "oaLogging"
except ImportError:
    _matrix_log = None
    LOGGING_BACKEND = "none"


def log(function, message, level="DEBUG"):
    """`matrix_log("ui", "importer", function, message, level)`, or nothing."""
    if _matrix_log is not None:
        _matrix_log("ui", "importer", function, message, level)


class Lifecycle:
    """start/stop/status/run_tests for a stateless importer.

    The importer holds no thread, socket or file between calls, so there is
    nothing to start or stop; these exist because the importer interface
    names them, and they say so rather than pretend to do something.
    """

    def __init__(self, status, self_test=None):
        if status not in STATUS_WORDS:
            raise ValueError(f"status {status!r} is not one of {STATUS_WORDS}")
        self._status = status
        self._self_test = self_test

    def start(self):
        """No-op by design: the importer is stateless between calls."""
        return None

    def stop(self):
        """No-op by design: the importer is stateless between calls."""
        return None

    def status(self):
        return self._status

    def run_tests(self):
        """True only for a live importer whose self test passes."""
        if self._status != "live" or self._self_test is None:
            return False
        try:
            return bool(self._self_test())
        except Exception as e:  # a self test that raises has failed, loudly
            log("run_tests", f"❌ importer self-test raised: {e}", "ERROR")
            return False


__all__ = ["MARKER_HEADERS", "STATUS_WORDS", "LOGGING_BACKEND", "log", "Lifecycle"]
