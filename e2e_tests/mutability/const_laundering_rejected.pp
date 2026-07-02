// Pins that a const cannot be laundered into a write-through position through
// call indirection:
//  - through a local fn-valued alias (`let f = mutate; f(o)`),
//  - through a higher-order call (`apply(mutate, o)` where apply calls f(v)),
//  - through a method receiver (`poke(c)` where poke's body calls a
//    self-mutating method on its parameter -- receivers are references),
//  - through a method's non-self parameter that forwards into a `ref(mut)`
//    position (`h.launder(c)`).
type P = { x: int32 }

type Counter = { n: int32 }

fun Counter.bump(self) {
    self.n = self.n + 1
}

type Helper = { dummy: int32 }

fun mutate(p: ref(mut(P))) {
    p.x = 42
}

fun apply(f, v) {
    f(v)
}

fun poke(c) {
    c.bump()
}

fun Helper.launder(self, q) {
    mutate(q)
}

fun main() {
    const o = P { x: 1 }
    let f = mutate
    f(o)

    apply(mutate, o)

    const c = Counter { n: 0 }
    poke(c)

    let h = Helper { dummy: 0 }
    h.launder(o)
}
