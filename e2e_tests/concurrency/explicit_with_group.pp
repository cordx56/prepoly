// Pins: the user-facing array form `with([a, b], ...)` acquires every cown in
// one address-ordered group, so two threads acquiring the same pair through
// arrays in opposite textual orders cannot deadlock and no update is lost.
type Counter = {
    value: int64
}

fun Counter.add(self, n: int64) { self.value = self.value + n }

fun main() {
    let a = Counter { value: 0 }
    let b = Counter { value: 0 }
    spawn(() -> {
        let i = 0
        while i < 1000 {
            with([a, b], (g) -> {
                a.add(1)
                b.add(1)
            })
            i = i + 1
        }
    })
    let j = 0
    while j < 1000 {
        with([b, a], (g) -> {
            b.add(1)
            a.add(1)
        })
        j = j + 1
    }
    sync()
    println("a = {a.value} b = {b.value}")
}
