// The spawner thread itself mutates the shared `counter` after the spawn, while
// the spawned task mutates it concurrently. The auto-acquire pass must guard the
// spawner's own accesses (not only the spawned closure body), or the two threads
// race and lose updates. With both sides lock-guarded the total is exactly
// 200000, deterministically across runs.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

// Explicit ref(mut): an unannotated mutated parameter would be a private copy.
fun work(c: ref(mut(Counter))) {
    let i = 0
    while i < 100000 {
        c.add(1)
        i = i + 1
    }
}

fun main() {
    let counter = Counter { value: 0 }
    spawn(() -> { work(counter) })
    work(counter)
    sync()
    println("value = {counter.value}")
}
