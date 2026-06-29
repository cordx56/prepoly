// A non-reference aggregate argument (record, tuple, nested array) is passed by
// deep copy: the callee mutates its own copy and the caller's value is unchanged.
type Box = { items: int32[] }

fun grow_box(b: Box) {
    b.items.push(9)
    println(b.items)
}

fun grow_nested(m: int32[][]) {
    m[0].push(99)
    println(m)
}

let b = Box { items: [1, 2] }
grow_box(b)
println(b.items)

let m = [[1], [2]]
grow_nested(m)
println(m)
