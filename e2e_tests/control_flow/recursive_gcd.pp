// Recursion with unannotated parameters, where both branches of the `if/else`
// diverge (each arm returns). The instance `gcd(int32, int32)` is inferred from
// the call site and the recursive call reads the provisional return type.
fun gcd(a, b) {
    if b == 0 {
        return a
    } else {
        return gcd(b, a % b)
    }
}

fun main() {
    println("{gcd(48, 36)}")
    println("{gcd(17, 5)}")
}
