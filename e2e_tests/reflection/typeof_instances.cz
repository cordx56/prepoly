// `typeof(x)` in a generic body must name each monomorphic instance's own
// argument type. All instances share one MIR body, so the name cannot be baked
// in at lowering: it is derived from the operand's per-instance type. The
// last-checked instantiation must not leak its name into the others.
type A = { v: int64 }
type S = | Red | Blue { n: int32 }

fun name(x) -> string {
    return typeof(x)
}

// Both a bare return and an interpolated use, across record/sum/string/int/float.
fun label(x) -> string {
    return "a {typeof(x)}"
}

println(name(A { v: 1 }))
println(name(S.Red))
println(name("plain"))
println(name(1))
println(name(1.5))
println(label(A { v: 2 }))
println(label("s"))
