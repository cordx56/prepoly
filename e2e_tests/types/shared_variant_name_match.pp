// Pins deterministic owning-sum resolution when two sums share variant names:
// a fully-exhaustive match on Big must be accepted on EVERY run. The owner
// used to be picked from HashMap iteration order, flipping between Small and
// Big and rejecting this program nondeterministically.
type Small =
    | X { a: int32 }
    | Y { a: int32 }

type Big =
    | X { a: int32 }
    | Y { a: int32 }
    | Z { a: int32 }

fun f(v: Big) -> int32 {
    return match v {
        X { a } => a,
        Y { a } => a,
        Z { a } => a,
    }
}

fun g(v: Small) -> int32 {
    return match v {
        X { a } => a,
        Y { a } => a,
    }
}

println(f(Big.Z { a: 7 }))
println(g(Small.X { a: 3 }))
