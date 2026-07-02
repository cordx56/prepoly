// A null-check narrowing does not survive a call when a closure of the same
// body assigns the narrowed variable: the call may run the closure and re-null
// it, so the use after the call must re-check for null.
fun main() {
    let x: int32? = 5
    let f = () -> {
        x = null
    }
    if x != null {
        f()
        let y: int32 = x
        println(y + 1)
    }
}
