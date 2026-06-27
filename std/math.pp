// Standard numeric helpers. `min`/`max`/`abs` are polymorphic over any type
// supporting `<`/`-`; the float routines wrap runtime primitives. Part of the
// implicit prelude.

// Absolute value. `x - x` yields a correctly typed zero so this works for any
// numeric type without a typed literal.
fun abs(x) {
    let zero = x - x
    if x < zero {
        return zero - x
    }
    return x
}

fun min(a, b) {
    if a < b {
        return a
    }
    return b
}

fun max(a, b) {
    if a > b {
        return a
    }
    return b
}

fun sqrt(x) -> float64 {
    return _float_sqrt(x)
}

fun floor(x) -> float64 {
    return _float_floor(x)
}

fun ceil(x) -> float64 {
    return _float_ceil(x)
}

fun pow(base, exp) -> float64 {
    return _float_pow(base, exp)
}
