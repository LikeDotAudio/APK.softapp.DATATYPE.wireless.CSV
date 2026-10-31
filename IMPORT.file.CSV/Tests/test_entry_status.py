# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
"""Entry.py says what it is: a STATUS word, and run_tests() never True for nothing."""
import Entry


def test_status_is_a_known_word_and_status_reports_it():
    assert Entry.STATUS in {"live", "unavailable"}
    assert Entry.status() == Entry.STATUS
    assert Entry.report_status() == Entry.STATUS


def test_run_tests_is_false_without_the_engine():
    if Entry.STATUS == "unavailable":
        assert Entry.run_tests() is False
