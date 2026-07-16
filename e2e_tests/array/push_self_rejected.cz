// Pushing an array into itself makes the element type infinitely recursive; the
// occurs check must reject it instead of leaving the element unbound (which let
// the call site mistype it and crashed the JIT).
fun grow(a: infer[]) {
    a.push(a)
}

fun main() {
    grow([1, 2, 3])
}
