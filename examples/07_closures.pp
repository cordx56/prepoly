// Closures capture their environment by reference, so captured variables stay
// shared and mutable across calls. Also shows method-chain and trailing-
// operator line continuation, and array destructuring.

fun make_accumulator(initial: int32) {
    let total = initial
    return (amount: int32) -> {
        total += amount
        return total
    }
}

fun main() {
    let acc = make_accumulator(100)
    println(acc(10))
    println(acc(20))
    println(acc(5))

    // A chain broken across lines (newline before `.` continues it).
    let result = [3, 1, 4, 1, 5, 9]
        .filter((x) -> x > 2)
        .map((x) -> x * 10)
        .fold(0, (a, b) -> a + b)
    println("chain result = {result}")

    // A binary operator at end of line continues onto the next.
    let total = 100 *
        2 +
        50
    println("total = {total}")

    let [first, second] = [10, 20]
    println("first={first}, second={second}")
}
