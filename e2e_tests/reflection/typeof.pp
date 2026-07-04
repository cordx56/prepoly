type Point = { x: int64, y: int64 }
type Shape = | Circle { r: float64 } | Square

fun describe(p: Point) -> string {
    return "a {typeof(p)} with fields:"
}

fun main() {
    const p = Point { x: 1, y: 2 }
    println(typeof(p))
    println(describe(p))
    const s = Shape.Circle { r: 2.0 }
    println(typeof(s))
    const n = 42
    println(typeof(n))
    const xs = [1, 2, 3]
    println(typeof(xs))
    for field in fields(p) {
        println("  {field}: (in {typeof(p)})")
    }
}
