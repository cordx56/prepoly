// float64 -> float32 loses precision, so it is explicit.
fun main() {
    let a: float64 = 1.5
    let b: float32 = a
    println(b)
}
