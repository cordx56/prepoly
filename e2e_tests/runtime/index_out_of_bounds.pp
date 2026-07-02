// An out-of-range index is a runtime error, not silent memory corruption. The
// program fails after the first line of output.
fun main() {
    let xs = [1, 2, 3]
    println(xs[2])
    println(xs[5])
}
