// Pins: an alias created through a function's return value before the spawn
// (`let d = id(counter)`) is treated as a handle to the cown, so the spawner's
// mutations through `d` are serialized under `counter`'s lock.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun id(x) {
    return x
}

fun main() {
    let counter = Counter { value: 0 }
    let d = id(counter)
    spawn(() -> {
        let i = 0
        while i < 100000 {
            counter.add(1)
            i = i + 1
        }
    })
    let j = 0
    while j < 100000 {
        d.add(1)
        j = j + 1
    }
    sync()
    println("counter = {counter.value} d = {d.value}")
}
