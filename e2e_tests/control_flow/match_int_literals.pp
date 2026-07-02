// `match` over integers with literal patterns, a negative literal, and the
// wildcard fallback (llm.md pattern-matching section).
fun describe(n: int32) -> string {
    return match n {
        0 => "zero",
        1 => "one",
        -1 => "minus one",
        _ => "many",
    }
}

fun main() {
    println(describe(0))
    println(describe(1))
    println(describe(-1))
    println(describe(42))
}
