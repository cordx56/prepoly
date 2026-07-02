// Pins: the task only READS the capture, but the spawner keeps WRITING it after
// the spawn. Freezing the capture (the old decision) let the writer race
// lock-free readers; the whole-function mutation rule now cowns it and guards
// both sides, so the reader can never observe a torn negative sum and the total
// is exact.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    let counter = Counter { value: 0 }
    spawn(() -> {
        let i = 0
        let sum = 0
        while i < 200000 {
            sum = sum + counter.value
            i = i + 1
        }
        println("reader saw sum >= 0: {sum >= 0}")
    })
    let j = 0
    while j < 200000 {
        counter.add(1)
        j = j + 1
    }
    sync()
    println("value = {counter.value}")
}
