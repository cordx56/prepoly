// A top-level binding's annotation fixes the global's type exactly like a
// function-local slot: the stored value coerces (int -> float; plain value
// into a nullable slot) and reads see the annotated representation.
let x: float64 = 1
let g: int32? = 5

fun main() {
    println(x)
    if g != null {
        println(g + 1)
    }
}
