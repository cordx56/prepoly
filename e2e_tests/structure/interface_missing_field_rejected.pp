// `type Student: Person` requires Student to include all of Person's fields
// (types.md); omitting one must be rejected.
type Person = {
    first_name: string
    last_name: string
}

type Student: Person = {
    first_name
    id
}

fun main() {
    let s = Student { first_name: "A", id: 1 }
}
