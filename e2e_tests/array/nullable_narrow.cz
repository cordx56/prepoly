// A nullable array narrowed by an `if a` guard must support every array
// operation on its inner `int32[]`. The guard proves non-null but does not
// retype the MIR local, so the value still carries the declared `int32[]?`;
// each op below previously failed to monomorphize, which silently dropped the
// whole module init and produced no output at all.
fun describe(a: int32[]?) {
    if a {
        println(a.len())
        for e in a {
            println(e)
        }
        println(a[0])
        a[0] = 100
        println(a[0])
        a.push(7)
        println(a.len())
        let last = a.pop()
        println(last)
    }
}

describe([1, 2, 3])
// The null arm takes neither branch: no output, no crash.
describe(null)
