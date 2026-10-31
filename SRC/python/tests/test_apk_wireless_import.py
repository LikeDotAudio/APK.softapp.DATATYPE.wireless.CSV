# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
"""The shared importer edge: the header row, and a lifecycle that cannot lie."""
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from apk_wireless_import import MARKER_HEADERS, STATUS_WORDS, LOGGING_BACKEND, Lifecycle, log  # noqa: E402


def test_the_header_row_is_the_frame_contract():
    assert MARKER_HEADERS == ["ZONE", "GROUP", "DEVICE", "NAME", "FREQ_MHZ", "PEAK"]


def test_a_stub_never_passes_its_tests():
    assert Lifecycle("stub", self_test=lambda: True).run_tests() is False


def test_an_unavailable_importer_never_passes():
    assert Lifecycle("unavailable", self_test=lambda: True).run_tests() is False


def test_a_live_importer_with_no_self_test_has_run_nothing():
    assert Lifecycle("live").run_tests() is False


def test_a_live_importer_passes_only_when_its_self_test_does():
    assert Lifecycle("live", self_test=lambda: True).run_tests() is True
    assert Lifecycle("live", self_test=lambda: False).run_tests() is False

    def boom():
        raise RuntimeError("engine gone")

    assert Lifecycle("live", self_test=boom).run_tests() is False


def test_status_reports_the_word_and_rejects_others():
    assert Lifecycle("stub").status() == "stub"
    assert set(STATUS_WORDS) == {"live", "stub", "unavailable"}
    with pytest.raises(ValueError):
        Lifecycle("Running with Rust Engine")


def test_start_and_stop_do_nothing():
    life = Lifecycle("live")
    assert life.start() is None and life.stop() is None


def test_log_never_raises_without_a_backend():
    assert LOGGING_BACKEND in {"oaLogging", "none"}
    log("t", "message")
