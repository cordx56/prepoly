// A method's fallibility must follow its DECLARED return, exactly as a free
// function's does: `-> T!` wraps plain returns in `Result.Ok` even when the body
// never builds an error itself. MIR judged a method by its body alone, so a
// declared-fallible method with no `error(..)` and no `!` was not marked
// fallible: it returned its bare value where the caller's `!` expected a Result.
// An array payload then read back as `never` ("indexing non-array `never`"), and
// a scalar payload silently read back as `null` -- a wrong answer, not an error.

type Box = { tag: int64 }

// Instance method, array payload, no error path.
fun Box.names(self) -> string[]! {
    let out: string[] = []
    out.push("a")
    out.push("b")
    return out
}

// Instance method, scalar payload, no error path (this one used to print `null`).
fun Box.label(self) -> string! {
    return "boxed"
}

// Static method, array payload, no error path.
fun Box.defaults() -> int64[]! {
    let out: int64[] = []
    out.push(7)
    return out
}

type Doc = | One { value: string } | Many { value: string[] }

// The same rule on a sum's method.
fun Doc.tags(self) -> string[]! {
    let out: string[] = []
    out.push("t")
    return out
}

fun main() {
    const b = Box { tag: 1 }
    const ns = b.names()!
    println("{len(ns)} {ns[0]} {ns[1]}")
    println(b.label()!)
    const ds = Box.defaults()!
    println(ds[0])
    const d = Doc.One { value: "x" }
    println(d.tags()![0])

    // A declared-fallible method that DOES error still reports it.
    match b.checked(-1) {
        Ok { value } => println("unexpected"),
        Err { error } => println("err: {error}"),
    }
    println(b.checked(3)!)
}

fun Box.checked(self, n: int64) -> int64! {
    if n < 0 {
        return error("negative")
    }
    return n
}
