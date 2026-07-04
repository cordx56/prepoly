// An annotated empty array literal is usable before any push: the checker's
// resolved type is seeded through to the back end, so a read-only use does
// not depend on deriving the element type from a later fill.
const top: int32[] = []

fun main() {
    let xs: int32[] = []
    println(xs.len())
    println(xs.pop())
    let ys: int32?[] = []
    println(ys.len())
    println(top.len())
}
