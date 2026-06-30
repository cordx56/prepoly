// Each spawned task mutates the shared counter through a local alias
// (`let a = counter; a.add(1)`), and the parent mutates it through its own alias.
// A purely syntactic mutation check would see the access rooted at `a`, classify
// `counter` read-only, freeze it, and race. Alias tracking must classify it a
// cown so every handle's access is lock-guarded; the total is exactly 300000.
type Counter = {
    value: int64
    add(self, n: int64) { self.value = self.value + n }
}

fun work(c) {
    let i = 0
    while i < 100000 {
        c.add(1)
        i = i + 1
    }
}

fun main() {
    let counter = Counter { value: 0 }
    let parent_handle = counter
    spawn(() -> {
        let a = counter
        work(a)
    })
    spawn(() -> {
        let b = counter
        work(b)
    })
    work(parent_handle)
    sync()
    println("value = {counter.value}")
}
