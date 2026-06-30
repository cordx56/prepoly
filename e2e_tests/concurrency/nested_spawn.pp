// A spawn nested inside another spawn mutates the shared counter from a third
// thread, while the parent mutates it too. The transform must descend into the
// outer spawn closure to promote and guard the inner spawn's captures, and must
// not mistake the inner closure's own loop counter for a capture of the outer
// one. With every access lock-guarded the total is exactly 200000.
type Counter = {
    value: int64
    add(self, n: int64) { self.value = self.value + n }
}

fun main() {
    let counter = Counter { value: 0 }
    spawn(() -> {
        spawn(() -> {
            let i = 0
            while i < 100000 {
                counter.add(1)
                i = i + 1
            }
        })
    })
    let j = 0
    while j < 100000 {
        counter.add(1)
        j = j + 1
    }
    sync()
    println("value = {counter.value}")
}
