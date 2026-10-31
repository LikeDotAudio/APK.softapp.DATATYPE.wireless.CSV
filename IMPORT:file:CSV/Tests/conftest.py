# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# Tests/conftest.py — make the importer importable where it actually lives.
#
# The folder is spelled `IMPORT:file:CSV`, which is not a Python identifier, so the
# package cannot be imported by name. The tests used to import `oaFileImportCSV.…`,
# the name it had in the desktop app it was lifted out of, which resolves nowhere in
# this tree. Instead the package root goes on sys.path and the tests import its
# modules (`FileReaders.from_csv_unknown`, `Entry`) directly.
#
# `apk_wireless_import` (the shared logging call + lifecycle) is this repository's
# own SRC/python/ package. An installed copy (`pip install -e SRC/python`) wins;
# otherwise the folder beside this one goes on sys.path. Nothing here reaches
# into another repository any more (it used to walk up to NODE.BareMetal for
# `importer_logging`).
import sys
from pathlib import Path

PACKAGE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PACKAGE))

try:
    import apk_wireless_import  # noqa: F401
except ImportError:
    sys.path.insert(0, str(PACKAGE.parent / "SRC" / "python"))
