// `match` over a sum type is checked for exhaustiveness (llm.md); a match
// missing a variant and lacking a wildcard must be rejected.
type Color =
    | Red
    | Green
    | Blue

fun name(c: Color) -> string {
    return match c {
        Color.Red => "red",
        Color.Green => "green",
    }
}

fun main() {
    println(name(Color.Blue))
}
