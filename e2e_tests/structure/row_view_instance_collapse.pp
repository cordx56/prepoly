// Three differently-shaped anonymous arguments collapse into ONE compiled
// instance of `display`: the derived row (first_name Guarded, last_name
// Required, both forced to string by the returns) fixes a single view type,
// so the extra fields and the absent guarded field never key new instances.
fun display(obj) -> string {
    if obj.first_name {
        return obj.first_name
    }
    return obj.last_name
}

fun main() {
    println(display({ first_name: "Ada", last_name: "Lovelace" }))
    println(display({ first_name: "Alan", last_name: "Turing", age: 41 }))
    println(display({ last_name: "Euler", country: "CH" }))
}
