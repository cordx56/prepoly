// Nullable types (`T?`) with narrowing, and the built-in Result type with the
// `error(...)` constructor and the `!` error-propagation operator.

fun parse_positive(s: string) {
    let n = int32.parse(s)!          // `!` returns early on a parse error
    if n < 0 {
        return error("negative number: {n}")
    }
    return n                          // implicitly wrapped in Result.Ok
}

// Returns the first even number, or null if there is none.
fun first_even(nums: int32[]) -> int32? {
    for n in nums {
        if n % 2 == 0 {
            return n
        }
    }
    return null
}

fun main() {
    for s in ["42", "-5", "abc"] {
        match parse_positive(s) {
            Ok { value } => println("parsed {s} -> {value}"),
            Err { error } => println("failed {s} -> {error}"),
        }
    }

    let found = first_even([1, 3, 4, 7])
    if found {
        // Inside the guard, `found` is narrowed from int32? to int32.
        println("first even = {found}")
    }
    let none = first_even([1, 3, 5])
    if !none {
        println("no even number found")
    }
}
