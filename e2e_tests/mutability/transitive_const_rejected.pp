// `record_mutator` takes a mutable reference (`ref(mut(Point))`) and writes
// through it; `forward` only passes its parameter on to `record_mutator`. That
// write-through position is interprocedural, so `forward`'s parameter also
// requires a mutable argument, and passing a `const` value through it must be
// rejected -- the const would otherwise be mutated at runtime through the
// forwarded reference. (An unannotated mutated parameter is a private deep copy,
// which a const argument may safely feed; only `ref(mut(..))` write-through is
// rejected.)
type Point = { x: int32 }

fun record_mutator(p: ref(mut(Point))) {
    p.x = 99
}

fun forward(p) {
    record_mutator(p)
}

fun main() {
    const origin = Point { x: 0 }
    forward(origin)
}
