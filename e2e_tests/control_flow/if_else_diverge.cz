// An `if/else` used as a statement where both arms return: the (unreachable)
// merge produces no value, so the lowering must not leave a dangling result.
fun classify(x: int32) -> int32 {
    if x == 0 {
        return 100
    } else {
        return 200
    }
}

fun main() {
    println("{classify(0)}")
    println("{classify(7)}")
}
