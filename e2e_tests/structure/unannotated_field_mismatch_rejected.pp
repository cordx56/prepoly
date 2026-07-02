// An unannotated declared field is pinned by the constructing literal: an
// `A { x: "..." }` instance must not satisfy a position requiring `{ x: int32 }`
// just because the declaration itself carries no annotation.
type A = { x }
type B = { x: int32 }

fun use_b(b: B) -> int32 {
    let n: int32 = b.x
    return n
}

fun main() {
    let a = A { x: "not an int" }
    println(use_b(a))
}
