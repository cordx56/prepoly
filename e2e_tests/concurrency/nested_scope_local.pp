// Pins: a local bound INSIDE an outer spawn closure and shared with a nested
// spawn is a capture of the INNER scope, not of the enclosing function. The old
// pass only collected function-level locals, so `local` was never promoted or
// guarded (rc race + data race). Each spawned closure body is now processed as
// its own scope with its own locals.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    spawn(() -> {
        let local = Counter { value: 0 }
        spawn(() -> {
            let i = 0
            while i < 100000 {
                local.add(1)
                i = i + 1
            }
        })
        let j = 0
        while j < 100000 {
            local.add(1)
            j = j + 1
        }
        sync()
        println("value = {local.value}")
    })
    sync()
}
