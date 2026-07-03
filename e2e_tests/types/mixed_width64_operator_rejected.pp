// int64 and uint64 have no value-preserving common type, so the operator has
// no applicable implicit conversion.
fun main() {
    let a: int64 = 1
    let u: uint64 = 2
    println(a + u)
}
