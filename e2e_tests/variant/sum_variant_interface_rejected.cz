// An interface on a sum type applies to every variant (llm.md: "This works for
// records and for every variant of a sum type"); a variant missing a required
// field must be rejected.
type Tagged = {
    tag: int32
}

type Event: Tagged =
    | Click { tag: int32, x: int32 }
    | Quit

fun main() {
    let e = Event.Quit
    println("ok")
}
