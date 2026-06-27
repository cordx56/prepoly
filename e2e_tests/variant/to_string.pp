type Point = {
    x: int32
    y: int32
}

type Shape =
    | Dot
    | Circle { center: Point, radius: int32 }

fun main() {
    const c = Shape.Circle { center: Point { x: 3, y: 4 }, radius: 5 }
    println("{c}")
    const d = Shape.Dot
    println("{d}")
}
