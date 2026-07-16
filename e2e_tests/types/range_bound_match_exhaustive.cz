// Pins that the exhaustiveness pass sees inside RANGE literal bounds: a
// non-exhaustive sum match hidden in `[0..(match ...)]` must be rejected (the
// shared expression walk used to skip Expr::Range entirely).
type S =
    | A { v: int32 }
    | B { v: int32 }

fun f(s: S) {
    println([0..(match s { A { v } => v })])
}

f(S.B { v: 3 })
