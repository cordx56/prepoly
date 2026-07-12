// Pins that a const cannot be laundered into a write-through position through
// call indirection:
//  - through a local fn-valued alias (`let f = mutate; f(o)`),
//  - through a higher-order call (`apply(mutate, o)` where apply calls f(v)).
//
// An unannotated parameter that mutates -- a self-mutating method on it, or
// forwarding it into a `ref(mut)` position -- is a PRIVATE DEEP COPY, so
// passing a const there is safe and is NOT rejected (see
// method_mutation_copies_param.pp).
type P = { x: int32 }

fun mutate(p: ref(mut(P))) {
    p.x = 42
}

fun apply(f, v) {
    f(v)
}

fun main() {
    const o = P { x: 1 }
    let f = mutate
    f(o)

    apply(mutate, o)
}
