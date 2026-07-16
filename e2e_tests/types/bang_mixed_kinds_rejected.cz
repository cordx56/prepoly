// One `!` span in a generic body, a nullable operand in one instantiation and
// a `Result` in another. The two propagate differently and the instantiations
// share a single MIR lowering, so no one shape is right: the checker must
// reject the program. (Left alone, the null-propagation shape was forced onto
// the Result instance, which then rendered the whole `Result.Ok {..}` where
// its payload was meant -- a silent wrong answer.)
fun might_fail(ok: bool) -> int32! {
    if ok { return 41 }
    return error("boom")
}

fun show(x) {
    return "value: {x!}"
}

fun main() {
    let n: int32? = 5
    println(show(n))
    let r = might_fail(true)
    println(show(r))
}
