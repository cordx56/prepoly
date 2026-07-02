// A null-check narrowing of a GLOBAL does not survive a call: any callee can
// assign the global, so the use after the call must re-check for null.
let g: int32? = 5

fun clear() {
    g = null
}

fun main() {
    if g != null {
        clear()
        println(g + 1)
    }
}
