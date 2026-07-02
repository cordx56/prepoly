// `T.from(v)` takes exactly one argument, and trailing arguments are still
// type-checked rather than silently dropped.
type Person = { name: string }

fun main() {
    let p = Person.from(1, nosuchvar)
    println(p == null)
}
