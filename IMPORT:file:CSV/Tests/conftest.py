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
# `importer_logging` lives in NODE.BareMetal/SRC/Core. It is found by walking up to
# the APK.audio checkout; APK_IMPORTER_LOGGING_DIR overrides that for a lone clone.
import os
import sys
from pathlib import Path

PACKAGE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PACKAGE))


def _logging_dir():
    if os.environ.get("APK_IMPORTER_LOGGING_DIR"):
        return Path(os.environ["APK_IMPORTER_LOGGING_DIR"])
    for up in PACKAGE.parents:
        cand = up / "PODS" / "POD.THICK" / "NODE.BareMetal" / "SRC" / "Core"
        if (cand / "importer_logging.py").exists():
            return cand
    return None


_dir = _logging_dir()
if _dir is not None:
    sys.path.insert(0, str(_dir))
