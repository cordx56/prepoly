// `record_mutator` mutates its parameter directly; `forward` only passes its
// parameter on to `record_mutator`. Mutability is interprocedural, so `forward`'s
// parameter is mutable too, and passing a `const` value through it must be
// rejected -- the const would otherwise be mutated at runtime through the
// forwarded reference.
type Point = { x: int32 }

fun record_mutator(p) {
    p.x = 99
}

fun forward(p) {
    record_mutator(p)
}

fun main() {
    const origin = Point { x: 0 }
    forward(origin)
}
