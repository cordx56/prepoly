// Pins: the cown stored as an array element before the spawn; the spawner's
// `arr[0].add(1)` aliases the task's capture and must acquire the capture's
// lock.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    let counter = Counter { value: 0 }
    let arr = [counter]
    spawn(() -> {
        let i = 0
        while i < 100000 {
            counter.add(1)
            i = i + 1
        }
    })
    let j = 0
    while j < 100000 {
        arr[0].add(1)
        j = j + 1
    }
    sync()
    println("value = {counter.value}")
}
