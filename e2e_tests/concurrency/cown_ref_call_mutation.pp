// A spawn closure mutates `counter` only through a plain function call
// `bump(counter)` -- a mutable-reference parameter, never `counter.method()` and
// never `counter = ...`. The ownership analysis must still see this as a mutation
// and cown (lock-guard) `counter`; otherwise the two threads race and lose
// updates. With locking + `sync()` the total is exactly 2000, deterministically.
type Counter = {
    value: int64
}

fun bump(c) {
    c.value = c.value + 1
}

fun main() {
    let counter = Counter { value: 0 }
    spawn(() -> {
        let i = 0
        while i < 1000 {
            bump(counter)
            i = i + 1
        }
    })
    spawn(() -> {
        let i = 0
        while i < 1000 {
            bump(counter)
            i = i + 1
        }
    })
    sync()
    println("value = {counter.value}")
}
