// A heterogeneous top-level array literal is a tuple in the light global pass
// exactly as in the full check, so function bodies read the same slot types the
// back end lays out.
let g = [1, "two"]

fun main() {
    println(g[0] + 1)
    println(g[1])
}
