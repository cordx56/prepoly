// A guarded field (every access dominated by a truthiness test of the field)
// may be absent from the argument: the guarded branch is not taken and the
// fallback runs, instead of a missing-field rejection.
fun greet(person) -> string {
    if person.nickname {
        return "hi {person.nickname}"
    }
    return "hello stranger"
}

fun main() {
    println(greet({ nickname: "Ace" }))
    println(greet({ name: "Bob" }))
}
