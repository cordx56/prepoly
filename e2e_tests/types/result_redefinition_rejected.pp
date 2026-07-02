// `Result` backs fallible returns and is built in; a user redefinition would
// silently vanish behind the builtin, so it is rejected.
type Result =
    | Good { v: int32 }
    | Bad

fun main() {
    println(1)
}
