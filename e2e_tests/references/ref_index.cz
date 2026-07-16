// Indexing through a reference yields a reference of the same kind: `ref(T[])[i]`
// is a `ref(T)` (read-only), `ref(mut(T[]))[i]` is a `ref(mut(T))` (assignable).
fun first(a: ref(int32[])) -> int32 {
    return a[0]
}

fun set_first(b: ref(mut(int32[]))) {
    b[0] = 99
}

let xs = [1, 2, 3]
println(first(xs))
set_first(xs)
println(xs)
