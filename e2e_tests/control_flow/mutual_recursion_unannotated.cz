// Mutual recursion without a return annotation has no sound provisional type
// to assume, so the typed backend rejects it with a hint naming the fix.
fun flip(n: int64) {
    if n == 0 {
        return 0
    }
    return flop(n - 1)
}

fun flop(n: int64) {
    if n == 0 {
        return 1
    }
    return flip(n - 1)
}

fun main() {
    println(flip(4))
}
