// `type B: A` requires B to provide every member of A, checked at compile time
// (llm.md interfaces section). A type missing a required method signature must
// be rejected.
type Showable = {
    to_string(self) -> string
}

type User: Showable = {
    name: string
}

fun main() {
    let u = User { name: "x" }
    println(u.name)
}
