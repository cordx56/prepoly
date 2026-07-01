// An `infer` parameter receives a read-only deep copy, so mutating it through
// its reference is rejected. A mutable copy needs `mut(T)`, a mutable reference
// `ref(mut(T))`, or the parameter can be left unannotated (inferred `mut`).
fun push_one(a: infer) {
    a.push(1)
}

fun main() {
    push_one([1, 2])
}
