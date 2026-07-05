// Error recovery: both bad statements are reported, each at the offending
// token's exact line:column, and the driver exits nonzero without running.
fun f() -> int32 {
    let x = )
    let y = ]
    return 0
}
fun main() {
    println(f())
}
