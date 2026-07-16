// Narrowing by a negated guard with an early return: after `if !x { return }`,
// `x` is narrowed from T? to T in the rest of the function (llm.md nullable
// section documents this form).
fun orders(x: int32?) -> int32 {
    if !x {
        return -1
    }
    return x * 10
}

fun main() {
    println(orders(4))
    println(orders(null))
}
