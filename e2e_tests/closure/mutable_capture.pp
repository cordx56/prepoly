// Closures capture their environment by mutable reference (llm.md): a closure
// that mutates a captured local is visible to the enclosing scope, and the
// state persists across calls.
fun main() {
    let total = 0
    let add = (amount: int32) -> {
        total += amount
        return total
    }
    println(add(5))
    println(add(7))
    println(total)
}
