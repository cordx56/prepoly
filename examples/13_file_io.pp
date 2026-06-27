// File I/O via the runtime primitives. `write_file` and `read_file` return a
// Result; we match on it explicitly.

fun main() {
    let path = "/tmp/prepoly_io_demo.txt"
    let content = "line one\nline two\nline three"

    match write_file(path, content) {
        Ok { value } => println("wrote {len(content)} bytes"),
        Err { error } => println("write failed: {error}"),
    }

    match read_file(path) {
        Ok { value } => {
            let lines = value.split("\n")
            println("read {len(lines)} lines:")
            for line in lines {
                println("  {line}")
            }
        },
        Err { error } => println("read failed: {error}"),
    }
}
