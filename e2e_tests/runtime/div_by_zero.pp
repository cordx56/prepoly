// Integer division by zero is a runtime error, not undefined behavior: the
// program aborts with a diagnostic instead of producing a garbage value.
fun main() {
    let a = 10
    let b = 0
    println(a / b)
}
