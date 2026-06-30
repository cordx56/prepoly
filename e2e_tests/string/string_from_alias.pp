// `string.from(s)` of a string is the identity -- its result aliases the argument.
// A bound alias must be retained so its later drop balances, otherwise the borrowed
// argument is over-released. Using one string value through several `string.from`
// conversions (directly and across calls) must not double-free it.

fun hash_len(key) {
    // `string.from(key)` aliases `key`; bound here and consumed by `_string_bytes`.
    let bytes = _string_bytes(string.from(key))
    return len(bytes)
}

fun main() {
    let k = "apple"
    // The same `k` flows into two conversions: each retains its alias independently.
    let a = string.from(k)
    let b = string.from(k)
    println("{a} {b}")

    // And across calls -- the pattern a hashed-key container produces internally.
    let x = hash_len(k)
    let y = hash_len(k)
    println("{x} {y}")
    println(k)
}
