type JsonValue =
    | Number { value }
    | Bool { value: bool }
    | String { value: string }
    | Object { value: JsonValue }

fun print_string(v: JsonValue) {
    match v {
        String { value } => { println("{value}") }
        _ => {}
    }
}

fun main() {
    const val = JsonValue.String { value: "test" }
    print_string(val)
    println("{val}")
}
