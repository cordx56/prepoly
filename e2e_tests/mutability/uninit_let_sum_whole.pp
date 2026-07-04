type Shape = | Point | Circle { r: float64 }
fun main() {
    let s: Shape
    if true {
        s = Shape.Circle { r: 2.0 }
    } else {
        s = Shape.Point
    }
    match s {
        Shape.Circle { r } => { println(r) }
        _ => { println("point") }
    }
}
