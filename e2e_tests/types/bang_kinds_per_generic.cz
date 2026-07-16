// Non-mixed generic `!` stays accepted: one generic unwraps only nullables
// (at different inner types), another only Results (different payloads).
type A = { v: int64 }

fun unwrap_null(x) {
    return x!
}

fun ok_int() -> int32! { return 7 }
fun ok_str() -> string! { return "hi" }

fun unwrap_res(x) {
    return x!
}

fun main() {
    let a: A? = A { v: 3 }
    let s: string? = "n"
    println(unwrap_null(a)!.v)
    println(unwrap_null(s)!)
    println(unwrap_res(ok_int())!)
    println(unwrap_res(ok_str())!)
}
