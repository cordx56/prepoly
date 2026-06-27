type Point = {
    x: int32
    y: int32
}

type Line = {
    a: Point
    b: Point
}

fun main() {
    const l = Line { a: Point { x: 1, y: 2 }, b: Point { x: 3, y: 4 } }
    println("{l.a.x} {l.b.y}")
    println("{l}")
}
