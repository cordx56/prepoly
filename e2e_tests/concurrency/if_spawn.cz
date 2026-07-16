// Pins: a spawn nested inside an if block must guard the spawner's accesses in
// the enclosing scope (the old per-list live set lost the cown when the block
// ended, racing the task against the loop after the if).
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    let counter = Counter { value: 0 }
    let go = true
    if go {
        spawn(() -> {
            let i = 0
            while i < 100000 {
                counter.add(1)
                i = i + 1
            }
        })
    }
    let j = 0
    while j < 100000 {
        counter.add(1)
        j = j + 1
    }
    sync()
    println("value = {counter.value}")
}
