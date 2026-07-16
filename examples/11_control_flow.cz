// Control flow: while, for-in, break, continue, nested if/else, early return.

// Collatz step count for n.
fun collatz_steps(n: int32) -> int32 {
    let count = 0
    let x = n
    while x != 1 {
        if x % 2 == 0 {
            x = x / 2
        } else {
            x = 3 * x + 1
        }
        count += 1
    }
    return count
}

fun first_multiple_of_7(limit: int32) -> int32 {
    let i = 1
    while i < limit {
        if i % 7 == 0 {
            return i
        }
        i += 1
    }
    return 0
}

fun main() {
    for n in [6, 7, 27] {
        println("collatz({n}) = {collatz_steps(n)} steps")
    }

    // `continue` skips, `break` stops.
    let sum = 0
    for n in [1, 2, 3, 4, 5, 6, 7, 8] {
        if n % 2 == 1 {
            continue
        }
        if n > 6 {
            break
        }
        sum += n
    }
    println("sum of evens up to 6 = {sum}")
    println("first multiple of 7 = {first_multiple_of_7(50)}")
}
