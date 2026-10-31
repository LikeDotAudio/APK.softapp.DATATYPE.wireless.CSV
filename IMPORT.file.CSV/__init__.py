# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# Package marker for the CSV importer.
#
# It carries no code on purpose. What it does carry is a name: without this
# file, `oaFileImportCSV` is a PEP 420 namespace package, and pytest names the
# suite under `Tests/` by its own basename — which collides with `Core/Tests/`
# and makes both uncollectable. Every one of the ten subpackages beside it
# already had an `__init__.py`; the package root was the one that did not.
