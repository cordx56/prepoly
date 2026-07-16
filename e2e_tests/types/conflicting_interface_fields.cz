// Two interfaces declare the same field `name` with different types. A type
// implementing both inherits an irreconcilable field type and must be rejected:
// a value reaching it through `Person` could be written through `Animal`, so the
// two field types must be invariant.
type Person = {
    name: int32
}

type Animal = {
    name: string
}

type Family: Person, Animal = {
    name
}

fun main() {
    let f = Family { name: 1 }
}
