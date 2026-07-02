// `read_file` of a nonexistent path returns a Result.Err that can be matched
// instead of crashing the program.
fun main() {
    match read_file("/nonexistent/prepoly_missing.txt") {
        Ok { value } => println("read: {value}"),
        Err { error } => println("error: {error}"),
    }
    println("still running")
}
