// A slice mutator has a fixed arity; extra arguments must be rejected, not
// silently dropped (`a.push(x, y)` previously discarded `y`, so `a.push(a, 2)`
// parsed as a self-push).
fun main() {
    let xs = [1]
    xs.push(2, 3)
}
