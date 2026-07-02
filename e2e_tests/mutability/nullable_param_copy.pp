// Pins that a nullable heap-aggregate parameter (`p: Point?`) is passed by
// deep copy exactly like the bare aggregate (`p: Point`): the `?` wrapper does
// not change the underlying value's kind, so mutating the parameter must never
// write through to the caller.
type Point = { x: int32 }

fun set_direct(p: Point) {
    p.x = 99
}

fun set_nullable(p: Point?) {
    if p {
        p.x = 99
    }
}

fun main() {
    let a = Point { x: 1 }
    set_direct(a)
    println(a.x)
    let b = Point { x: 1 }
    set_nullable(b)
    println(b.x)
}
