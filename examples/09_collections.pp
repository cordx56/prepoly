// Standard-library collection and math helpers (the implicit prelude). These
// are written in Prepoly and reached through UFCS: `arr.map(f)` resolves to
// the free function `map(arr, f)`.

fun main() {
    let nums = [5, 3, 8, 1, 9, 2]
    println("sorted   = {nums.sort()}")
    println("reversed = {nums.reverse()}")
    println("doubled  = {nums.map((x) -> x * 2)}")
    println("evens    = {nums.filter((x) -> x % 2 == 0)}")
    println("sum      = {nums.fold(0, (a, b) -> a + b)}")
    println("contains 8 = {nums.contains(8)}")
    println("slice 1..4 = {nums.slice(1, 4)}")

    println("abs(-7)  = {abs(-7)}")
    println("min/max  = {min(3, 9)} / {max(3, 9)}")
    println("sqrt 2.0 = {sqrt(2.0)}")

    let chain = [1, 2, 3, 4, 5, 6]
        .filter((x) -> x % 2 == 0)
        .map((x) -> x * x)
        .fold(0, (a, b) -> a + b)
    println("sum of squares of evens = {chain}")
}
