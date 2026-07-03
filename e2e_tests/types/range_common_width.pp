// A range's element type is the bounds' common integer type, exactly like a
// binary operator's operands: a literal bound adapts to the other bound's
// width, so counting over an int64 length runs at int64 -- the counter must
// not wrap at the literal's default width.
fun inner_product(a, b) {
    let result = 0.0
    for i in [0..a.len()] {
        result += a[i] * b[i]
    }
    return result
}
println(inner_product([1, 2, 3], [3, 2, 1]))
println(inner_product([1.5, 2.0], [2.0, 4.0]))

// Mixed annotated widths meet at the wider type (int64 here); the loop
// variable carries it, so adding near INT64_MAX stays in range.
let n: int32 = 1
let m: int64 = 4
for i in [n..m] {
    println(i + 9223372036854775800)
}

// The counter crosses the int32 boundary: a counter re-derived at the
// literal bound's default width would wrap to -2147483648 and never
// terminate.
let big: int64 = 2147483650
let count = 0
for i in [2147483646..big] {
    println(i)
    count += 1
}
println(count)

// Unsigned bounds: the literal lo adapts to uint64, and the counter's
// increment validates at the adapted kind.
let v: uint64 = 3
for i in [0..v] {
    println(i)
}

// The materialized range value carries the same element type.
let xs = [2147483646..big]
println(xs)
println(xs[3] + 1)
