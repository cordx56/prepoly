// Positive control for the match-divergence rule: a literal match WITH a
// catch-all arm is guaranteed to run one arm, so all-arms-return still counts
// as diverging and the function is accepted. Also pins an exhaustive sum match
// without a wildcard (the exhaustiveness pass vouches for its coverage).
type Dir =
    | Up { }
    | Down { }

fun pick(x: int32) -> int32 {
    match x {
        1 => { return 10 },
        _ => { return 20 },
    }
}

fun step(d: Dir) -> int32 {
    match d {
        Up { } => { return 1 },
        Down { } => { return -1 },
    }
}

println(pick(1))
println(pick(3))
println(step(Dir.Up { }))
println(step(Dir.Down { }))
