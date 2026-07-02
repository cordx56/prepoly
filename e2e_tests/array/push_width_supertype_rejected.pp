// Array elements are invariant storage: a record that structurally satisfies
// the element type but has a wider layout (extra fields, different offsets)
// must not be pushed -- reading it back would reinterpret the wider layout.
type P = { x: int32 }
type Q = { y: string, x: int32 }

fun main() {
    let a = [P { x: 1 }]
    a.push(Q { y: "s", x: 2 })
    println(a[1].x)
}
