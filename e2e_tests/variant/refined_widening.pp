// A sum value refined by an unannotated variant field (e.g. `S<Untagged.value=..>`)
// is usable where the bare nominal `S` is required: `describe` takes a bare `S`.
type S =
    | Tagged { value: string }
    | Untagged { value }

fun describe(s: S) -> string {
    return match s {
        Tagged { value } => value,
        _ => "untagged",
    }
}

fun main() {
    println(describe(S.Tagged { value: "hi" }))
    println(describe(S.Untagged { value: 5 }))
}
