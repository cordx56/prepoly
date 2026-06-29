// `T.from(v)` yields `T?`; `if let` narrows it and the branch is decided per
// monomorphized argument type: a value with all of Person's fields takes the
// then-arm, a value missing a field takes the else-arm. No runtime reflection --
// each call's concrete argument type fixes the branch.
type Person = { first_name: string, last_name: string }

fun greet(obj) {
    if let p = Person.from(obj) {
        return "hi {p.first_name} {p.last_name}"
    } else {
        return "not a person"
    }
}

println(greet({ first_name: "Ada", last_name: "Lovelace" }))
println(greet({ first_name: "x" }))
println(greet({ nope: 1 }))
