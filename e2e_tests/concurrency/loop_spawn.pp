// Pins: a spawn nested inside a while loop shares the counter with everything
// the function does afterwards. The old pass kept a per-statement-list live set,
// so the access AFTER the loop was never guarded and raced the two tasks.
// With whole-function guarding the total is exactly 300000.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    let counter = Counter { value: 0 }
    let k = 0
    while k < 2 {
        spawn(() -> {
            let i = 0
            while i < 100000 {
                counter.add(1)
                i = i + 1
            }
        })
        k = k + 1
    }
    let j = 0
    while j < 100000 {
        counter.add(1)
        j = j + 1
    }
    sync()
    println("value = {counter.value}")
}
