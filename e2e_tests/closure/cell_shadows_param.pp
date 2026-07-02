// Pins cell promotion for a `let` that shadows a parameter name: the binding a
// closure captures and mutates is the shadowing local, which must be
// heap-promoted to a shared cell even though a parameter shares its name
// (parameters themselves are never promoted). The old name-keyed exclusion
// stripped the promotion, so the closure mutated a private copy and f printed
// 10 instead of 11.
fun f(x: int32) -> int32 {
    let x = 10
    let inc = () -> {
        x = x + 1
    }
    inc()
    return x
}

fun main() {
    println(f(1))
}
