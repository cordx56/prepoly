// Pins the for-loop element write-back on every edge that ends an iteration:
// a body reassigning the loop variable mutates the array in place even when
// the iteration ends via `continue` or `break`, not only on the fall-through
// tail (which used to leave `continue`d iterations unwritten).
fun main() {
    let a = [1, 2, 3]
    for e in a {
        e *= 2
        continue
    }
    println(a)

    let b = [1, 2, 3]
    for e in b {
        e *= 2
        if e > 2 {
            continue
        }
    }
    println(b)

    let c = [1, 2, 3]
    for e in c {
        e = e + 10
        break
    }
    println(c)
}
