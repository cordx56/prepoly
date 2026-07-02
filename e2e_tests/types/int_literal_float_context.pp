// An integer literal in a float context becomes a float, and any integer width
// can index an array -- inside function bodies exactly as at the top level.
fun main() {
    let x: float64 = 1
    println(x)
    let f: float64 = 2.5
    println(f * 2)
    let xs = [10, 20, 30]
    let i: int32 = 1
    println(xs[i])
}
