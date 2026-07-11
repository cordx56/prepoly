// `typeof(x).method()` dispatches statically through a span-keyed channel, so
// one generic body cannot route the call to a DIFFERENT type per instantiation
// -- even when both candidate types define the method. The checker must reject
// the program rather than let the last-checked instantiation's type dispatch
// every call.
type A = { v: int64 }
type B = { v: int64 }
fun A.origin() -> A { return A { v: 1 } }
fun B.origin() -> B { return B { v: 2 } }
fun mk(x) { return typeof(x).origin() }
println(mk(A { v: 0 }).v)
println(mk(B { v: 0 }).v)
