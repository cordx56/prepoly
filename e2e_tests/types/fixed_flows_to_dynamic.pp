// A fixed-length array is usable where a dynamic array of the same element is
// required (types.md: "the length is extra static information").
fun total(xs: int32[]) -> int32 {
    let sum = 0
    for x in xs {
        sum += x
    }
    return sum
}

fun main() {
    const fixed = [1, 2, 3]
    println(total(fixed))
}
