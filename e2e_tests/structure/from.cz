// `T.from(v)` is a fallible structural conversion: it yields `T?`, the record when
// the (monomorphized) value has all of `T`'s fields, else null. Narrow it with
// `if let` to use the converted record.
type Point = { x: int32, y: int32 }

let big = { x: 1, y: 2, z: 3 }
if let p = Point.from(big) {
    println(p.x)
    println(p.y)
}
