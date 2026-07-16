// Callee mutates its param: entry deep copy means each call sees the pristine
// literal, and the caller's value is untouched.
fun bump(a) -> int32 {
    a[0] = a[0] + 100
    return a[0]
}
println(bump([1, 2, 3]))
println(bump([1, 2, 3]))

// Local mutation through a binding: must keep per-evaluation identity.
let m = [10, 20]
m[0] = 11
println(m)

// Escape into a mutable global structure, then write through the alias.
let holder = [[0]]
fun stash(x) {
    holder[0] = x
}
stash([5, 6])
holder[0][0] = 55
println(holder[0])
stash([5, 6])
println(holder[0])

// Same literal in a loop, read-only callee: values stay pristine.
fun sum(a) {
    let t = 0
    for i in [0..a.len()] {
        t += a[i]
    }
    return t
}
let total = 0
for _i in [0..3] {
    total += sum([7, 8])
}
println(total)

// Return of a literal-derived value: not promoted, still correct.
fun make() {
    return [1, 1]
}
let k = make()
k[0] = 3
println(k)
