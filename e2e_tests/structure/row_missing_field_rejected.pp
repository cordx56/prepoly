// A field the callee's row requires (an unguarded use) must be present on an
// anonymous argument; the error is reported AT THE VALUE, naming the field,
// instead of at a span inside the callee body.
fun display(obj) -> string {
    return obj.first_name + " " + obj.last_name
}

fun main() {
    println(display({ first_name: "Ada" }))
}
