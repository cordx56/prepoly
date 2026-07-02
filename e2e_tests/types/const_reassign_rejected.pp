// Reassigning a `const` binding is a compile-time error (book: "const pi = ...
// // immutable binding; reassigning is a compile error").
fun main() {
    const x = 1
    x = 2
    println(x)
}
