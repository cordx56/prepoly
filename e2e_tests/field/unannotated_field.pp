// An unannotated (dynamic) field has a per-value type: two `Box` values may carry
// `value` at different types without coupling the field to a single type.
type Box = { value }

fun main() {
    let a = Box { value: 42 }
    let b = Box { value: "hello" }
    println("{a.value}")
    println("{b.value}")
}
