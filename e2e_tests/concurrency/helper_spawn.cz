// Pins: the spawn happens inside a HELPER function; the caller keeps mutating
// the object it passed in. The interprocedural summary ("start spawns a task
// capturing parameter 0") makes the caller promote its local to a cown and
// guard its own accesses, so caller and task serialize (exactly 200000).
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

// Explicit ref(mut): the spawned closure mutates the captured parameter, and an
// unannotated mutated parameter would be a private copy of the caller's counter.
fun start(c: ref(mut(Counter))) {
    spawn(() -> {
        let i = 0
        while i < 100000 {
            c.add(1)
            i = i + 1
        }
    })
}

fun main() {
    let counter = Counter { value: 0 }
    start(counter)
    let j = 0
    while j < 100000 {
        counter.add(1)
        j = j + 1
    }
    sync()
    println("value = {counter.value}")
}
