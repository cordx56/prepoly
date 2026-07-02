// Pins that mutating a local alias of an unannotated parameter counts as
// mutating the parameter: `let q = p` binds another handle to the same heap
// value, so `q.x = 99` must classify `p` as mutated and make it a private deep
// copy -- the caller's record (even a const one) is unchanged.
type Point = {
    x: int32
}

fun sneak(p) {
    let q = p
    q.x = 99
}

fun main() {
    let pt = Point { x: 1 }
    sneak(pt)
    println(pt.x)

    const cpt = Point { x: 2 }
    sneak(cpt)
    println(cpt.x)
}
