// An anonymous argument passes as its VIEW: rendering the parameter inside
// the callee shows exactly the row's fields (here only `name`), not the
// caller's extra fields. This is the one observable difference of the view
// conversion and is pinned as specified behavior.
fun show(obj) -> string {
    let name: string = obj.name
    println(obj)
    return name
}

fun main() {
    println(show({ name: "Ada", age: 36 }))
}
