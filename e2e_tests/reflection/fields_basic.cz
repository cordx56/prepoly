type Point = {
    x: int64,
    y: int64,
}

// Read side: the loop variable decays to the field name as a string; v[field]
// projects the field.
fun dump(p: Point) {
    for field in fields(p) {
        println("{field} = {p[field]}")
    }
}

// Write side: field-wise construction of an uninitialized let through the loop.
fun scaled(base: Point, k: int64) -> Point {
    let ret: Point
    for field in fields(ret) {
        ret[field] = base[field] * k
    }
    return ret
}

fun main() {
    const p = Point { x: 3, y: 4 }
    dump(p)
    const s = scaled(p, 10)
    println(s)
}
