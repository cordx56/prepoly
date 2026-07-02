// Calling a non-function field through call syntax is a type error naming the
// field's actual type, not a deferred runtime failure.
type A = {
    x: int32
}

fun main() {
    let a = A { x: 1 }
    println(a.x(4))
}
