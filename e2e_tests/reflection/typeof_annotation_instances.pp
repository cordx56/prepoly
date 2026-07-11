// A `let y: typeof(x) = x` slot in a generic body must take each instance's
// own type. The annotation's resolved type is span-keyed while all instances
// share one MIR body, so on a cross-instance disagreement the checker drops
// the recorded slot type and the binding is inferred per instance from its
// initializer -- the string instance must not be read through the record's
// layout (or vice versa).
type A = { v: int64 }

fun dup(x) {
    let y: typeof(x) = x
    return y
}

println(dup("s"))
println(dup(A { v: 7 }).v)
println(dup(42))
