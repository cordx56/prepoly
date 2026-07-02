// Pins that monomorphized instance symbols are collision-resistant: type names
// containing `_` must not let (A_B, C) and (A, B_C) share one instance. A
// display-joined symbol keyed both calls as `get__A_B_C`, so the second call
// reused a body typed int32 for a string field (garbage int on the JIT, 0 on
// the interpreter).
type A_B =
    | P { v: int32 }

type C =
    | Q { v: int32 }

type A =
    | R { v: int32 }

type B_C =
    | S { v: string }

fun get(x, y) {
    return y.v
}

fun main() {
    println(get(A_B.P { v: 1 }, C.Q { v: 7 }))
    println(get(A.R { v: 1 }, B_C.S { v: "hello" }))
}
