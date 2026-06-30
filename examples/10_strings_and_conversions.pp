// String utilities (prelude) and explicit numeric conversions. Numeric operators
// implicitly convert mixed numeric operands to a common type; use `from`/`parse`
// when the conversion itself is the operation you want to spell out.

fun main() {
    let csv = "alice,bob,carol"
    let names = csv.split(",")
    println("count = {len(names)}")
    println("joined = {names.join(" | ")}")
    let upper = "hello".to_upper()
    println("upper = {upper}")
    println("trimmed = '{"   spaced   ".trim()}'")
    println("starts = {"prepoly".starts_with("pre")}")
    println("replace = {"a-b-c".replace("-", "+")}")

    // Conversions: parse returns a Result, `from` converts between numbers.
    let n = int32.parse("123")!
    let f = float64.from(n) + 0.5
    println("n = {n}, f = {f}")
    println("string.from = {string.from(42)} and {string.from(true)}")
}
