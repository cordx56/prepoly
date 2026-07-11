// Importing one name from a module must not leak the module's other public
// names into bare scope: only the prelude is import-free. `parse` lives in
// the data.json library and is not imported here, so calling it is an error
// naming the module that has it.
import data.json.{ JsonValue }

let v = parse("[1]")
println(v)
