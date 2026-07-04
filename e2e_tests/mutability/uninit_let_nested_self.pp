type Inner = { v: int64 }
type Outer = { a: int64, inner: Inner }

fun Outer.blank() -> Outer {
    let ret: Self
    ret.a = 1
    ret.inner = Inner { v: 42 }
    return ret
}

fun main() {
    let o: Outer
    o.a = 5
    o.inner = Inner { v: 6 }
    println(o.inner.v)
    const b = Outer.blank()
    println(b.inner.v)
}
