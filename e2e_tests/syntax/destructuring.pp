// Array/tuple destructuring in `let` (llm.md declarations): positional
// bindings from an array literal and from a heterogeneous tuple.
fun main() {
    let [a, b] = [10, 20]
    println(a + b)
    let [n, s] = [1, "one"]
    println("{n} is {s}")
}
