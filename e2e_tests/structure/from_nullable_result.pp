// A fallible `T.from` feeding an `if let` whose else propagates an error: the
// success arm returns the narrowed record (Ok-wrapped), the failure arm errors.
type Person = { first_name: string, last_name: string }

fun show_person(obj) {
    if let person = Person.from(obj) {
        return person
    } else {
        error("type mismatch!")!
    }
}

println(show_person({ first_name: "a", last_name: "b" }))
println(show_person({ first_name: "a" }))
