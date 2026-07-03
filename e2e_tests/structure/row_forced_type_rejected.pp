// A use inside the callee pins a field's type (here `let age: int32 = p.age`);
// an anonymous argument whose field cannot flow into that forced type is
// rejected at the value, naming both types.
fun describe(p) -> string {
    let age: int32 = p.age
    return "age {age}"
}

fun main() {
    println(describe({ age: "twenty" }))
}
