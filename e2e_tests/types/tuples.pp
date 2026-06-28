// A bracket literal with differing element types is a fixed-length tuple: it can
// be destructured by an array pattern, indexed at a constant position, annotated
// as `[T0, T1, ...]`, and printed. A homogeneous literal stays an array.
fun head(t: [int32, string]) -> int32 {
    return t[0]
}

// Top-level destructure into globals.
let [n, label] = [7, "seven"]
println(n)
println(label)

// Index a literal directly.
println([1, "s"][0])
println([1, "s"][1])

// A 3-tuple, mixed kinds, printed whole and read by position.
let triple = [42, "hi", true]
println(triple)
println(triple[2])

// Annotated tuple parameter.
println(head([9, "x"]))

// A homogeneous bracket literal is still an array (summed, not a tuple).
let nums = [1, 2, 3, 4]
let total = 0
for x in nums {
    total = total + x
}
println(total)
