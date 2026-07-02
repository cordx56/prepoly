// Pins: a closure-variable spawn's read-only capture is still promoted to an
// atomic-count owner before the spawn. Both threads churn references to the
// shared record through a helper (bind/drop aliases in a loop); with the old
// non-atomic count this double-freed the object mid-run.
type Node = {
    value: int64
}

fun touch(n) {
    let a = n
    let b = a
    return b.value
}

fun main() {
    let n = Node { value: 42 }
    let task = () -> {
        let i = 0
        let s = 0
        while i < 200000 {
            s = s + touch(n)
            i = i + 1
        }
        println("task sum ok = {s > 0}")
    }
    spawn(task)
    let j = 0
    let s = 0
    while j < 200000 {
        s = s + touch(n)
        j = j + 1
    }
    sync()
    println("main sum ok = {s > 0} value = {n.value}")
}
