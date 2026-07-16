type Vec2 = { x: float64, y: float64 }

fun Vec2.origin() -> Vec2 {
    return Vec2 { x: 0.0, y: 0.0 }
}

fun main() {
    const v = Vec2 { x: 1.0, y: 2.0 }
    // typeof(v) as a static receiver: call Vec2's static method through a value.
    const o = typeof(v).origin()
    println("{o.x} {o.y}")
    // typeof on a primitive value routes a numeric conversion.
    const n: int64 = 300
    const back = typeof(n).from(3.9)!
    println(back)
    // and still works as a string in value position.
    println("v is a {typeof(v)}, o is a {typeof(o)}")
}
