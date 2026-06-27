type Point = {
    x: int32
    y: int32
}

type Shape =
    | Circle { center: Point, radius: int32 }

fun main() {
    const c = Shape.Circle { center: Point { x: 7, y: 9 }, radius: 2 }
    match c {
        Circle { center, radius } => {
            println("{center.x} {center.y} {radius}")
            println("{center}")
        }
    }
}
