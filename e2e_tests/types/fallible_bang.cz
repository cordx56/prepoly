// `T!` is the fallible Result type: success payload `T`, error payload inferred
// from the `error(...)` sites. A bare `return v` wraps as `Ok { value: v }`.
fun parse_pos(n: int32) -> int32! {
    if n < 0 {
        error(n)!
    }
    return n
}

match parse_pos(7) {
    Ok { value } => println("ok {value}"),
    Err { error } => println("err {error}"),
}
match parse_pos(-3) {
    Ok { value } => println("ok {value}"),
    Err { error } => println("err {error}"),
}
