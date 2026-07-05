// A dependency's syntax error is located in the imported file (its own
// line:column), not at this import statement.
import dep_with_error.{ helper }
println(helper())
