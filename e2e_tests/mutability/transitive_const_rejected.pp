// `record_mutator` takes a mutable reference (`ref(mut(Point))`) and writes
// through it; `forward` passes its own `ref(mut(Point))` parameter on, so the
// caller's value is mutated through two reference hops. Passing a `const`
// into that chain must be rejected. An UNANNOTATED `forward(p)` would instead
// deep-copy `p` at entry (forwarding into a mutating position counts as
// mutation), making a const argument safe -- so only the explicit reference
// chain can reject here.
type Point = { x: int32 }

fun record_mutator(p: ref(mut(Point))) {
    p.x = 99
}

fun forward(p: ref(mut(Point))) {
    record_mutator(p)
}

fun main() {
    const origin = Point { x: 0 }
    forward(origin)
}
