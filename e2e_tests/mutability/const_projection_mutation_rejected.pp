// Pins const propagation through derived types and projections:
//  - a const bound from a function CALL carries the callee's return type, so a
//    self-mutating method on it is rejected (`c.bump()`),
//  - aliasing a const record's FIELD shares the same heap value, so mutating
//    through the alias is rejected (`a.v = 99`),
//  - a const array's ELEMENT type is followed through indexing, so a
//    self-mutating method on `arr[0]` is rejected.
type Counter = { n: int32 }

fun Counter.bump(self) {
    self.n = self.n + 1
}

fun make() -> Counter {
    return Counter { n: 0 }
}

type Inner = { v: int32 }
type Outer = { inner: Inner }

fun main() {
    const c = make()
    c.bump()

    const o = Outer { inner: Inner { v: 1 } }
    let a = o.inner
    a.v = 99

    const arr = [Counter { n: 0 }]
    arr[0].bump()
}
