// An inferred return re-wrapped as `T?` because the body has a `return null`
// builds `?` over the still-open parameter variable. When that variable later
// pins to a nullable type the return would read `T??`, a type the language
// cannot distinguish from `T?` (there is one `null`). The solver collapses it,
// so passing an already-nullable value through the generic function is legal
// and the value survives the round trip.
fun pick(x) {
    if 1 == 2 {
        return null
    }
    return x
}

fun main() {
    let present: string? = "hi"
    // The closure keeps `x` open at re-elaboration; `f`'s argument pins it.
    let f = (s) -> pick(s)

    let a: string? = f(present)
    if let v = a {
        println(v)
    } else {
        println("absent")
    }

    let missing: string? = null
    let b: string? = f(missing)
    if let v = b {
        println(v)
    } else {
        println("absent")
    }
}
