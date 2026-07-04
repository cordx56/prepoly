// An empty array literal whose element type nothing ever fixes is rejected
// with a diagnostic naming the binding and the enclosing function.
fun main() {
    let xs = []
    println(xs.len())
}
