// Right field name, wrong field type: the field's type must be checked, otherwise
// a string is read as an int32 in the unboxed back end.
fun use_int(p: anonymous { v: int32 }) -> int32 {
    return p.v + 1
}

fun main() {
    println(use_int({ v: "not a number" }))
}
