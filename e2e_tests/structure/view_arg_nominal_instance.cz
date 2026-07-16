// view_args is a span SET united across a generic's instantiations, so the
// nominal instance's argument is wrapped in a RecordView too and takes the
// identity pass-through. That alias must be retained like the fresh view the
// binding otherwise owns -- without it the nominal record was over-released
// and the program crashed AFTER printing the right answers.
type N = { a: int64, c: string }

fun takes(p) -> int64 {
    return p.a
}

fun mid(x) {
    return takes(x)
}

fun main() {
    println(mid({ a: 1, b: 2 }))
    println(mid(N { a: 3, c: "z" }))
    println(mid({ a: 4, d: 9.5 }))
}
