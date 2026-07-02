// A function-typed record field is callable through call syntax (a.func(4)
// calls the stored closure; methods of the same name take precedence), and the
// field can be bound to a local and called indirectly. Pins both the annotated
// field ((int32)->int32) and the checker/back-end plumbing for closures stored
// into record fields.
type A = {
    func: (int32) -> int32
}

fun main() {
    const f = (x) -> 2 * x
    println(f(8))
    const a = A { func: (x) -> x - 2 }
    println(a.func(4))
    const g = a.func
    println(g(4))
}
