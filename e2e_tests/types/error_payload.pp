// `error(x)` accepts any payload, not just strings; the payload comes back out
// of `Err { error }` with its type intact.
fun pick(n: int32) -> int32! {
    if n < 0 {
        return error(n)
    }
    return n * 2
}

fun main() {
    match pick(-7) {
        Ok { value } => println("ok {value}"),
        Err { error } => println("bad input {error}"),
    }
    println(pick(21))
}
