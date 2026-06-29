// An anonymous-struct parameter must be field-checked: a value missing the
// required fields must be rejected rather than slipping through as compatible
// with everything.
fun take(p: anonymous { x: int32, y: int32 }) -> int32 {
    return p.x + p.y
}

fun main() {
    println(take({ a: "hello" }))
}
