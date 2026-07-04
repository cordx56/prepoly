type Vec2 = { x: float64, y: float64 }

fun swap(v: Vec2) -> Vec2 {
    // typeof(v) in type position: y is declared with v's type.
    let y: typeof(v) = Vec2 { x: v.y, y: v.x }
    return y
}

fun main() {
    const v = Vec2 { x: 1.0, y: 2.0 }
    // Declare a same-typed local via typeof.
    let w: typeof(v) = Vec2 { x: 9.0, y: 8.0 }
    println("{w.x} {w.y}")
    const s = swap(v)
    println("{s.x} {s.y}")
    // typeof in a nullable position.
    let maybe: typeof(v)? = null
    maybe = v
    if let got = maybe {
        println("{got.x}")
    }
}
