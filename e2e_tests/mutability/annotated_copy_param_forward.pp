// Pins that a bare-record parameter (`q: Point`, deep-copied at entry) that is
// forwarded into a `ref(mut(..))` position does NOT become a write-through
// position itself: the callee mutates f's private copy, so a const argument is
// accepted and the caller's value is unchanged. (The write-through fixpoint
// and the back ends' copy decision must agree on this; a divergent copy
// predicate used to reject the const spuriously.)
type Point = {
    x: int32
}

fun g(p: ref(mut(Point))) {
    p.x = 9
}

fun f(q: Point) {
    g(q)
}

fun main() {
    const c = Point { x: 1 }
    f(c)
    println(c.x)
}
