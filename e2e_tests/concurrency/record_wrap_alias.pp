// Pins: the cown is also reachable through a wrapping record built before the
// spawn; the spawner's writes through `w.c` must be guarded under `counter`'s
// lock (the object the task locks), not under `w`'s.
type Counter = {
    value: int64
}
type Wrap = {
    c: Counter
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    let counter = Counter { value: 0 }
    let w = Wrap { c: counter }
    spawn(() -> {
        let i = 0
        while i < 100000 {
            counter.add(1)
            i = i + 1
        }
    })
    let j = 0
    while j < 100000 {
        w.c.add(1)
        j = j + 1
    }
    sync()
    println("value = {counter.value}")
}
