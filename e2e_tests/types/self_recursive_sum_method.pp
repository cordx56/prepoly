// A SELF-RECURSIVE method on a wide sum must compile -- and in bounded time.
//
// A sum's methods are lowered into every variant's table, so one `e.render()`
// resolves to one candidate per variant and the checker re-elaborates each. The
// recursion guard keyed those elaborations by the `Sum.Variant` qualifier the
// call resolved through, so a recursive call re-entered through a variant not yet
// on the stack and the guard never fired: every level re-elaborated the body once
// per not-yet-entered variant, and the work grew FACTORIALLY in the variant
// count. On a sum this wide with a body this heavy, the compiler hung -- no
// diagnostic, no progress. The guard is now keyed by the receiver TYPE, so a
// recursive call falls back to the declared return exactly as it does for a free
// function.

type Node =
    | Text { value: string }
    | Int { value: int64 }
    | Real { value: float64 }
    | Flag { value: bool }
    | Tag { value: string }
    | Pair { value: Node[] }
    | List { value: Node[] }

fun Node.render(self) -> string {
    match self {
        Node.Text { value } => { return _quoted(value) }
        Node.Int { value } => { return "{value}" }
        Node.Real { value } => { return "{value}" }
        Node.Flag { value } => {
            if value { return "true" }
            return "false"
        }
        Node.Tag { value } => { return "<" + value + ">" }
        Node.Pair { value } => {
            let out = "("
            for e in value {
                out = out + e.render()
            }
            return out + ")"
        }
        Node.List { value } => {
            let out = "["
            let first = true
            for e in value {
                if !first { out = out + "," }
                out = out + e.render()
                first = false
            }
            return out + "]"
        }
    }
}

// A helper the recursive method calls, which itself builds an inferred `[]`.
fun _quoted(s: string) -> string {
    let out = ""
    for c in s.chars() {
        out = out + c
    }
    return "\"" + out + "\""
}

fun main() {
    let inner: Node[] = []
    inner.push(Node.Int { value: 1 })
    inner.push(Node.Text { value: "hi" })
    inner.push(Node.Flag { value: true })

    let outer: Node[] = []
    outer.push(Node.List { value: inner })
    outer.push(Node.Tag { value: "b" })

    const doc = Node.Pair { value: outer }
    println(doc.render())
}
