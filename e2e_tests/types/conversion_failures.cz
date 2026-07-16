// Fallible conversions return Result: uint8.from out of range and int32.parse
// of a non-number are Err; in-range conversions unwrap with `!` (llm.md
// numbers section: `uint8.from(300)` is an Err).
fun try_u8(n: int32) -> string {
    return match uint8.from(n) {
        Ok { value } => "ok {value}",
        Err { error } => "err {error}",
    }
}

fun main() {
    println(try_u8(200))
    println(try_u8(300))
    match int32.parse("abc") {
        Ok { value } => println("parsed {value}"),
        Err { error } => println("parse failed: {error}"),
    }
    let n = int32.parse("123")!
    println(n + 1)
}
