// Importing one name from a nested std module must not leak the module's
// other public names into bare scope: only the prelude is import-free.
// `parse` lives in std.data.json and is not imported here, so calling it is
// an error naming the module that has it.
import std.data.json.{ JsonValue }

let v = parse("[1]")
println(v)
