// int64 and uint64 have no value-preserving common type, so a range mixing
// them requires an explicit conversion on one bound.
let a: int64 = 1
let b: uint64 = 5
for i in [a..b] {
    println(i)
}
