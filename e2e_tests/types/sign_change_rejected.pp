// A signed value never flows into an unsigned type implicitly.
fun main() {
    let a: int32 = -1
    let u: uint32 = a
    println(u)
}
