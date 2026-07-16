// An uncalled member access is a compile-time presence test, so one generic
// body serves a record, a string and an array: each arm is checked and emitted
// only for the instantiation whose receiver reaches it.
type Segments = { parts: string[] }

const SEP = "/"

fun describe(s) -> string {
    if s.parts {
        return "record: {s.parts.join(SEP)}"
    } else if s.split {
        return "string: {s.split(SEP).join(SEP)}"
    } else {
        return "array: {s.join(SEP)}"
    }
}

println(describe(Segments { parts: ["usr", "lib"] }))
println(describe("a/b/c"))
println(describe(["x", "y"]))

// The presence value itself: a member the class carries decays to its own name,
// one it does not have is null.
const xs = [1, 2]
println(xs.push)
println(xs.no_such_member == null)
println("abc".split)
println("abc".no_such_member == null)
