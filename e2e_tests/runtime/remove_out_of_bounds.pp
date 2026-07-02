// Pins that removing at an out-of-range index halts with a runtime error
// (matching the interpreter) instead of silently returning a zero element
// and continuing. The line after the remove must never run.
fun main() {
    let a = [1, 2]
    let v = a.remove(5)
    println(v)
    println("unreachable")
}
