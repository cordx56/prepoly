// A module reads its OWN `_PATH`, never the importer's: the constant is injected
// per module at load time, and its leading `_` keeps it private to this file.
fun lib_path() -> string {
    return _PATH
}
