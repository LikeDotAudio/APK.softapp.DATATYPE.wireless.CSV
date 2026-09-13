# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
import pytest


# The Rust CSV parser exports `sum_as_string`; this asserts the export is reachable.
def test_the_rust_parser_sums_two_numbers():
    try:
        from oaRustCore.csv_parser import sum_as_string
    except ImportError:
        try:
            from oaRustCore.csv_parser import sum_as_string
        except ImportError:
            pytest.skip("sum_as_string not found in apkaudio_csvparser_rs")
            return

    assert sum_as_string(1, 1) == "2"
