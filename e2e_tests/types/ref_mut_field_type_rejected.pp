// A `ref(mut(T))` parameter exposes the underlying record's fields: storing a
// value of the wrong type through the reference must be a type error, not a
// silently unchecked write (which would corrupt the unboxed layout).
type Point = { x: int32 }

fun poke(p: ref(mut(Point))) {
    p.x = "not an int"
}

fun main() {
    let pt = Point { x: 1 }
    poke(pt)
    println(pt.x + 1)
}
