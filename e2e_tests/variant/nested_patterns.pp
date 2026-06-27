type Shape =
    | Circle { r: int32 }
    | Square { s: int32 }

fun describe(sh: Shape) -> string {
    return match sh {
        Circle { r: 0 } => "empty",
        Circle { r } => "circle",
        Square { s } => "square",
    }
}

fun main() {
    println(describe(Shape.Circle { r: 0 }))
    println(describe(Shape.Circle { r: 5 }))
    println(describe(Shape.Square { s: 3 }))
}
